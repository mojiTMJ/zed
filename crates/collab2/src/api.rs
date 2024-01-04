use crate::{
    auth,
    db::{User, UserId},
    rpc, AppState, Error, Result,
};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Path, Query},
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::response::ErasedJson;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tracing::instrument;

pub fn routes(rpc_server: Arc<rpc::Server>, state: Arc<AppState>) -> Router<Body> {
    Router::new()
        .route("/user", get(get_authenticated_user))
        .route("/users/:id/access_tokens", post(create_access_token))
        .route("/panic", post(trace_panic))
        .route("/rpc_server_snapshot", get(get_rpc_server_snapshot))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(state))
                .layer(Extension(rpc_server))
                .layer(middleware::from_fn(validate_api_token)),
        )
}

pub async fn validate_api_token<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let token = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            Error::Http(
                StatusCode::BAD_REQUEST,
                "missing authorization header".to_string(),
            )
        })?
        .strip_prefix("token ")
        .ok_or_else(|| {
            Error::Http(
                StatusCode::BAD_REQUEST,
                "invalid authorization header".to_string(),
            )
        })?;

    let state = req.extensions().get::<Arc<AppState>>().unwrap();

    if token != state.config.api_token {
        Err(Error::Http(
            StatusCode::UNAUTHORIZED,
            "invalid authorization token".to_string(),
        ))?
    }

    Ok::<_, Error>(next.run(req).await)
}

#[derive(Debug, Deserialize)]
struct AuthenticatedUserParams {
    github_user_id: Option<i32>,
    github_login: String,
    github_email: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthenticatedUserResponse {
    user: User,
    metrics_id: String,
}

async fn get_authenticated_user(
    Query(params): Query<AuthenticatedUserParams>,
    Extension(app): Extension<Arc<AppState>>,
) -> Result<Json<AuthenticatedUserResponse>> {
    let user = app
        .db
        .get_or_create_user_by_github_account(
            &params.github_login,
            params.github_user_id,
            params.github_email.as_deref(),
        )
        .await?
        .ok_or_else(|| Error::Http(StatusCode::NOT_FOUND, "user not found".into()))?;
    let metrics_id = app.db.get_user_metrics_id(user.id).await?;
    return Ok(Json(AuthenticatedUserResponse { user, metrics_id }));
}

#[derive(Deserialize, Debug)]
struct CreateUserParams {
    github_user_id: i32,
    github_login: String,
    email_address: String,
    email_confirmation_code: Option<String>,
    #[serde(default)]
    admin: bool,
    #[serde(default)]
    invite_count: i32,
}

#[derive(Serialize, Debug)]
struct CreateUserResponse {
    user: User,
    signup_device_id: Option<String>,
    metrics_id: String,
}

#[derive(Debug, Deserialize)]
struct Panic {
    version: String,
    release_channel: String,
    backtrace_hash: String,
    text: String,
}

#[instrument(skip(panic))]
async fn trace_panic(panic: Json<Panic>) -> Result<()> {
    tracing::error!(version = %panic.version, release_channel = %panic.release_channel, backtrace_hash = %panic.backtrace_hash, text = %panic.text, "panic report");
    Ok(())
}

async fn get_rpc_server_snapshot(
    Extension(rpc_server): Extension<Arc<rpc::Server>>,
) -> Result<ErasedJson> {
    Ok(ErasedJson::pretty(rpc_server.snapshot().await))
}

#[derive(Deserialize)]
struct CreateAccessTokenQueryParams {
    public_key: String,
    impersonate: Option<String>,
}

#[derive(Serialize)]
struct CreateAccessTokenResponse {
    user_id: UserId,
    encrypted_access_token: String,
}

async fn create_access_token(
    Path(user_id): Path<UserId>,
    Query(params): Query<CreateAccessTokenQueryParams>,
    Extension(app): Extension<Arc<AppState>>,
) -> Result<Json<CreateAccessTokenResponse>> {
    let user = app
        .db
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow!("user not found"))?;

    let mut user_id = user.id;
    if let Some(impersonate) = params.impersonate {
        if user.admin {
            if let Some(impersonated_user) = app.db.get_user_by_github_login(&impersonate).await? {
                user_id = impersonated_user.id;
            } else {
                return Err(Error::Http(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("user {impersonate} does not exist"),
                ));
            }
        } else {
            return Err(Error::Http(
                StatusCode::UNAUTHORIZED,
                "you do not have permission to impersonate other users".to_string(),
            ));
        }
    }

    let access_token = auth::create_access_token(app.db.as_ref(), user_id).await?;
    let encrypted_access_token =
        auth::encrypt_access_token(&access_token, params.public_key.clone())?;

    Ok(Json(CreateAccessTokenResponse {
        user_id,
        encrypted_access_token,
    }))
}