use crate::{
    wasm_host::{wit, WasmState},
    ExtensionIndex, ExtensionIndexEntry, ExtensionIndexLanguageEntry, ExtensionManifest,
    ExtensionStore, GrammarManifestEntry,
};
use collections::BTreeMap;
use fs::FakeFs;
use futures::FutureExt;
use gpui::{Context, TestAppContext};
use language::{LanguageMatcher, LanguageRegistry};
use node_runtime::FakeNodeRuntime;
use project::Project;
use serde_json::json;
use settings::SettingsStore;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use theme::ThemeRegistry;
use util::http::{FakeHttpClient, Response};
use wasmtime::Store;
use wasmtime_wasi::preview2::WasiView;

#[gpui::test]
async fn test_extension_store(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let store = SettingsStore::test(cx);
        cx.set_global(store);
        theme::init(theme::LoadThemes::JustBase, cx);
    });

    let fs = FakeFs::new(cx.executor());
    let http_client = FakeHttpClient::with_200_response();

    fs.insert_tree(
        "/the-extension-dir",
        json!({
            "installed": {
                "zed-monokai": {
                    "extension.json": r#"{
                        "id": "zed-monokai",
                        "name": "Zed Monokai",
                        "version": "2.0.0"
                    }"#,
                    "themes": {
                        "monokai.json": r#"{
                            "name": "Monokai",
                            "author": "Someone",
                            "themes": [
                                {
                                    "name": "Monokai Dark",
                                    "appearance": "dark",
                                    "style": {}
                                },
                                {
                                    "name": "Monokai Light",
                                    "appearance": "light",
                                    "style": {}
                                }
                            ]
                        }"#,
                        "monokai-pro.json": r#"{
                            "name": "Monokai Pro",
                            "author": "Someone",
                            "themes": [
                                {
                                    "name": "Monokai Pro Dark",
                                    "appearance": "dark",
                                    "style": {}
                                },
                                {
                                    "name": "Monokai Pro Light",
                                    "appearance": "light",
                                    "style": {}
                                }
                            ]
                        }"#,
                    }
                },
                "zed-ruby": {
                    "extension.json": r#"{
                        "id": "zed-ruby",
                        "name": "Zed Ruby",
                        "version": "1.0.0",
                        "grammars": {
                            "ruby": {"repository": "", "commit": ""},
                            "embedded_template": {"repository": "", "commit": ""},
                        }
                    }"#,
                    "grammars": {
                        "ruby.wasm": "",
                        "embedded_template.wasm": "",
                    },
                    "languages": {
                        "ruby": {
                            "config.toml": r#"
                                name = "Ruby"
                                grammar = "ruby"
                                path_suffixes = ["rb"]
                            "#,
                            "highlights.scm": "",
                        },
                        "erb": {
                            "config.toml": r#"
                                name = "ERB"
                                grammar = "embedded_template"
                                path_suffixes = ["erb"]
                            "#,
                            "highlights.scm": "",
                        }
                    },
                }
            }
        }),
    )
    .await;

    let mut expected_index = ExtensionIndex {
        extensions: [
            (
                "zed-ruby".into(),
                ExtensionManifest {
                    id: "zed-ruby".into(),
                    name: "Ruby".into(),
                    version: "1.0.0".into(),
                    description: None,
                    authors: Vec::new(),
                    repository: None,
                    themes: Vec::new(),
                    lib: Default::default(),
                    languages: vec!["languages/ruby".into(), "languages/erb".into()],
                    grammars: [
                        ("ruby".into(), GrammarManifestEntry::default()),
                        ("erb".into(), GrammarManifestEntry::default()),
                    ]
                    .into_iter()
                    .collect(),
                    language_servers: BTreeMap::default(),
                }
                .into(),
            ),
            (
                "zed-monokai".into(),
                ExtensionManifest {
                    id: "zed-monokai".into(),
                    name: "Zed Monokai".into(),
                    version: "2.0.0".into(),
                    description: None,
                    authors: vec![],
                    repository: None,
                    themes: vec![
                        "themes/monokai.json".into(),
                        "themes/monokai-pro.json".into(),
                    ],
                    lib: Default::default(),
                    languages: Vec::new(),
                    grammars: BTreeMap::default(),
                    language_servers: BTreeMap::default(),
                }
                .into(),
            ),
        ]
        .into_iter()
        .collect(),
        languages: [
            (
                "ERB".into(),
                ExtensionIndexLanguageEntry {
                    extension: "zed-ruby".into(),
                    path: "languages/erb".into(),
                    grammar: Some("embedded_template".into()),
                    matcher: LanguageMatcher {
                        path_suffixes: vec!["erb".into()],
                        first_line_pattern: None,
                    },
                },
            ),
            (
                "Ruby".into(),
                ExtensionIndexLanguageEntry {
                    extension: "zed-ruby".into(),
                    path: "languages/ruby".into(),
                    grammar: Some("ruby".into()),
                    matcher: LanguageMatcher {
                        path_suffixes: vec!["rb".into()],
                        first_line_pattern: None,
                    },
                },
            ),
        ]
        .into_iter()
        .collect(),
        themes: [
            (
                "Monokai Dark".into(),
                ExtensionIndexEntry {
                    extension: "zed-monokai".into(),
                    path: "themes/monokai.json".into(),
                },
            ),
            (
                "Monokai Light".into(),
                ExtensionIndexEntry {
                    extension: "zed-monokai".into(),
                    path: "themes/monokai.json".into(),
                },
            ),
            (
                "Monokai Pro Dark".into(),
                ExtensionIndexEntry {
                    extension: "zed-monokai".into(),
                    path: "themes/monokai-pro.json".into(),
                },
            ),
            (
                "Monokai Pro Light".into(),
                ExtensionIndexEntry {
                    extension: "zed-monokai".into(),
                    path: "themes/monokai-pro.json".into(),
                },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let language_registry = Arc::new(LanguageRegistry::test());
    let theme_registry = Arc::new(ThemeRegistry::new(Box::new(())));
    let node_runtime = FakeNodeRuntime::new();

    let store = cx.new_model(|cx| {
        ExtensionStore::new(
            PathBuf::from("/the-extension-dir"),
            fs.clone(),
            http_client.clone(),
            node_runtime.clone(),
            language_registry.clone(),
            theme_registry.clone(),
            cx,
        )
    });

    cx.executor().run_until_parked();
    store.read_with(cx, |store, _| {
        let index = &store.extension_index;
        assert_eq!(index.extensions, expected_index.extensions);
        assert_eq!(index.languages, expected_index.languages);
        assert_eq!(index.themes, expected_index.themes);

        assert_eq!(
            language_registry.language_names(),
            ["ERB", "Plain Text", "Ruby"]
        );
        assert_eq!(
            theme_registry.list_names(false),
            [
                "Monokai Dark",
                "Monokai Light",
                "Monokai Pro Dark",
                "Monokai Pro Light",
                "One Dark",
            ]
        );
    });

    fs.insert_tree(
        "/the-extension-dir/installed/zed-gruvbox",
        json!({
            "extension.json": r#"{
                "id": "zed-gruvbox",
                "name": "Zed Gruvbox",
                "version": "1.0.0"
            }"#,
            "themes": {
                "gruvbox.json": r#"{
                    "name": "Gruvbox",
                    "author": "Someone Else",
                    "themes": [
                        {
                            "name": "Gruvbox",
                            "appearance": "dark",
                            "style": {}
                        }
                    ]
                }"#,
            }
        }),
    )
    .await;

    expected_index.themes.insert(
        "Gruvbox".into(),
        ExtensionIndexEntry {
            extension: "zed-gruvbox".into(),
            path: "themes/gruvbox.json".into(),
        },
    );

    store.update(cx, |store, cx| store.reload(cx));

    cx.executor().run_until_parked();
    store.read_with(cx, |store, _| {
        let index = &store.extension_index;
        assert_eq!(index.extensions, expected_index.extensions);
        assert_eq!(index.languages, expected_index.languages);
        assert_eq!(index.themes, expected_index.themes);

        assert_eq!(
            theme_registry.list_names(false),
            [
                "Gruvbox",
                "Monokai Dark",
                "Monokai Light",
                "Monokai Pro Dark",
                "Monokai Pro Light",
                "One Dark",
            ]
        );
    });

    let prev_fs_metadata_call_count = fs.metadata_call_count();
    let prev_fs_read_dir_call_count = fs.read_dir_call_count();

    // Create new extension store, as if Zed were restarting.
    drop(store);
    let store = cx.new_model(|cx| {
        ExtensionStore::new(
            PathBuf::from("/the-extension-dir"),
            fs.clone(),
            http_client.clone(),
            node_runtime.clone(),
            language_registry.clone(),
            theme_registry.clone(),
            cx,
        )
    });

    cx.executor().run_until_parked();
    store.read_with(cx, |store, _| {
        assert_eq!(store.extension_index, expected_index);
        assert_eq!(
            language_registry.language_names(),
            ["ERB", "Plain Text", "Ruby"]
        );
        assert_eq!(
            language_registry.grammar_names(),
            ["embedded_template".into(), "ruby".into()]
        );
        assert_eq!(
            theme_registry.list_names(false),
            [
                "Gruvbox",
                "Monokai Dark",
                "Monokai Light",
                "Monokai Pro Dark",
                "Monokai Pro Light",
                "One Dark",
            ]
        );

        // The on-disk manifest limits the number of FS calls that need to be made
        // on startup.
        assert_eq!(fs.read_dir_call_count(), prev_fs_read_dir_call_count);
        assert_eq!(fs.metadata_call_count(), prev_fs_metadata_call_count + 2);
    });

    store.update(cx, |store, cx| {
        store.uninstall_extension("zed-ruby".into(), cx)
    });

    cx.executor().run_until_parked();
    expected_index.extensions.remove("zed-ruby");
    expected_index.languages.remove("Ruby");
    expected_index.languages.remove("ERB");

    store.read_with(cx, |store, _| {
        assert_eq!(store.extension_index, expected_index);
        assert_eq!(language_registry.language_names(), ["Plain Text"]);
        assert_eq!(language_registry.grammar_names(), []);
    });
}

#[gpui::test]
async fn test_extension_store_with_language_servers(cx: &mut TestAppContext) {
    init_test(cx);

    let fs = FakeFs::new(cx.executor());
    let http_client = FakeHttpClient::create(|_| async move {
        Ok(Response::new(
            json!([
                {
                    "tag_name": "something",
                    "prerelease": false,
                    "tarball_url": "",
                    "zipball_url": "",
                    "assets": [
                        {
                            "name": "ok",
                            "browser_download_url": "http://the-download-url.example.com"
                        }
                    ]
                }
            ])
            .to_string()
            .into(),
        ))
    });

    fs.insert_tree(
        "/the-extension-dir",
        json!({
            "installed": {
                "language_server_example": {
                    "extension.json": r#"{
                        "id": "language_server_example",
                        "name": "An Extension With Language Servers",
                        "version": "2.0.0",
                        "lib": {
                            "path": "extension.wasm"
                        },
                        "language_servers": {
                            "example": {
                                "name": "the-language-server",
                                "language": "Rust"
                            }
                        }
                    }"#,
                }
            }
        }),
    )
    .await;

    fs.insert_file(
        "/the-extension-dir/installed/language_server_example/extension.wasm",
        compile_example_extension("language_server_example"),
    )
    .await;

    let language_registry = Arc::new(LanguageRegistry::test());
    let theme_registry = Arc::new(ThemeRegistry::new(Box::new(())));
    let node_runtime = FakeNodeRuntime::new();

    let store = cx.new_model(|cx| {
        ExtensionStore::new(
            PathBuf::from("/the-extension-dir"),
            fs.clone(),
            http_client.clone(),
            node_runtime,
            language_registry.clone(),
            theme_registry.clone(),
            cx,
        )
    });

    cx.executor().run_until_parked();
    let extension = store.read_with(cx, |store, _| store.wasm_extensions[0].clone());

    fs.insert_tree(
        "/the-project-dir",
        json!({
            ".tool-versions": "rust 1.73.0",
        }),
    )
    .await;

    let project = Project::test(fs.clone(), ["/the-project-dir".as_ref()], cx).await;
    let worktree = project.read_with(cx, |project, cx| {
        project.worktrees().next().unwrap().read(cx).snapshot()
    });

    let config = extension
        .0
        .language_servers
        .values()
        .next()
        .unwrap()
        .clone();
    let command = extension
        .1
        .call(
            |extension: &mut wit::Extension, store: &mut Store<WasmState>| {
                async move {
                    let resource = store.data_mut().table().push(worktree).unwrap();
                    let command = extension
                        .call_get_language_server_command(
                            store,
                            &wit::LanguageServerConfig {
                                name: config.name,
                                language_name: config.language,
                            },
                            resource,
                        )
                        .await;
                    command
                }
                .boxed()
            },
        )
        .await
        .unwrap()
        .unwrap();

    dbg!(&command);
}

#[gpui::test]
async fn test_extension_store_with_real_extension(cx: &mut TestAppContext) {
    init_test(cx);

    let fs = FakeFs::new(cx.executor());
    let http_client = FakeHttpClient::create(|_| async move {
        Ok(Response::new(
            json!([
                {
                    "tag_name": "something",
                    "prerelease": false,
                    "tarball_url": "",
                    "zipball_url": "",
                    "assets": [
                        {
                            "name": "ok",
                            "browser_download_url": "http://the-download-url.example.com"
                        }
                    ]
                }
            ])
            .to_string()
            .into(),
        ))
    });

    let gleam_extension_dir = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../extensions/gleam"
    ))
    .canonicalize()
    .unwrap();
    compile_extension(&gleam_extension_dir);

    fs.insert_tree("/the-extension-dir", json!({ "installed": {} }))
        .await;
    fs.insert_tree_from_real_fs("/the-extension-dir/installed/gleam", gleam_extension_dir)
        .await;

    let language_registry = Arc::new(LanguageRegistry::test());
    let theme_registry = Arc::new(ThemeRegistry::new(Box::new(())));
    let node_runtime = FakeNodeRuntime::new();

    let store = cx.new_model(|cx| {
        ExtensionStore::new(
            PathBuf::from("/the-extension-dir"),
            fs.clone(),
            http_client.clone(),
            node_runtime,
            language_registry.clone(),
            theme_registry.clone(),
            cx,
        )
    });

    cx.executor().run_until_parked();
    let extension = store.read_with(cx, |store, _| store.wasm_extensions[0].clone());

    fs.insert_tree(
        "/the-project-dir",
        json!({
            ".tool-versions": "rust 1.73.0",
        }),
    )
    .await;

    let project = Project::test(fs.clone(), ["/the-project-dir".as_ref()], cx).await;
    let worktree = project.read_with(cx, |project, cx| {
        project.worktrees().next().unwrap().read(cx).snapshot()
    });

    let config = extension
        .0
        .language_servers
        .values()
        .next()
        .unwrap()
        .clone();
    let command = extension
        .1
        .call(
            |extension: &mut wit::Extension, store: &mut Store<WasmState>| {
                async move {
                    let resource = store.data_mut().table().push(worktree).unwrap();
                    let command = extension
                        .call_get_language_server_command(
                            store,
                            &wit::LanguageServerConfig {
                                name: config.name,
                                language_name: config.language,
                            },
                            resource,
                        )
                        .await;
                    command
                }
                .boxed()
            },
        )
        .await
        .unwrap()
        .unwrap();

    dbg!(&command);
}

fn compile_extension(extension_dir_path: &Path) {
    dbg!(extension_dir_path);

    let output = std::process::Command::new("cargo")
        .args(["component", "build", "--target-dir"])
        .arg(extension_dir_path.join("target"))
        .current_dir(&extension_dir_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "failed to build component {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let name = "zed_gleam";

    let mut wasm_path = PathBuf::from(extension_dir_path);
    wasm_path.extend(["target", "wasm32-wasi", "debug", name]);
    wasm_path.set_extension("wasm");

    std::fs::rename(wasm_path, extension_dir_path.join("extension.wasm")).unwrap();
}

fn compile_example_extension(name: &str) -> Vec<u8> {
    let mut example_dir = std::env::current_dir().unwrap();
    example_dir.extend(["example_extensions", name]);

    let output = std::process::Command::new("cargo")
        .args(["component", "build"])
        .current_dir(&example_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "failed to build component {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut wasm_path = example_dir;
    wasm_path.extend(["target", "wasm32-wasi", "debug", name]);
    wasm_path.set_extension("wasm");

    std::fs::read(&wasm_path).unwrap()
}

fn init_test(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let store = SettingsStore::test(cx);
        cx.set_global(store);
        theme::init(theme::LoadThemes::JustBase, cx);
        Project::init_settings(cx);
    });
}
