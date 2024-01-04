use std::ops::Range;

use crate::{
    color::Color,
    geometry::{
        deserialize_vec2f,
        rect::RectF,
        vector::{vec2f, Vector2F},
    },
    json::ToJson,
    platform::CursorStyle,
    scene::{self, CornerRadii, CursorRegion, Quad},
    AnyElement, Element, SizeConstraint, ViewContext,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
pub struct ContainerStyle {
    #[serde(default)]
    pub margin: Margin,
    #[serde(default)]
    pub padding: Padding,
    #[serde(rename = "background")]
    pub background_color: Option<Color>,
    #[serde(rename = "overlay")]
    pub overlay_color: Option<Color>,
    #[serde(default)]
    pub border: Border,
    #[serde(default)]
    #[serde(alias = "corner_radius")]
    pub corner_radii: CornerRadii,
    #[serde(default)]
    pub shadow: Option<Shadow>,
    #[serde(default)]
    pub cursor: Option<CursorStyle>,
}

impl ContainerStyle {
    pub fn fill(color: Color) -> Self {
        Self {
            background_color: Some(color),
            ..Default::default()
        }
    }

    pub fn additional_length(&self) -> f32 {
        self.padding.left
            + self.padding.right
            + self.border.width * 2.
            + self.margin.left
            + self.margin.right
    }
}

pub struct Container<V> {
    child: AnyElement<V>,
    style: ContainerStyle,
}

impl<V> Container<V> {
    pub fn new(child: AnyElement<V>) -> Self {
        Self {
            child,
            style: Default::default(),
        }
    }

    pub fn with_style(mut self, style: ContainerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_margin_top(mut self, margin: f32) -> Self {
        self.style.margin.top = margin;
        self
    }

    pub fn with_margin_bottom(mut self, margin: f32) -> Self {
        self.style.margin.bottom = margin;
        self
    }

    pub fn with_margin_left(mut self, margin: f32) -> Self {
        self.style.margin.left = margin;
        self
    }

    pub fn with_margin_right(mut self, margin: f32) -> Self {
        self.style.margin.right = margin;
        self
    }

    pub fn with_horizontal_padding(mut self, padding: f32) -> Self {
        self.style.padding.left = padding;
        self.style.padding.right = padding;
        self
    }

    pub fn with_vertical_padding(mut self, padding: f32) -> Self {
        self.style.padding.top = padding;
        self.style.padding.bottom = padding;
        self
    }

    pub fn with_uniform_padding(mut self, padding: f32) -> Self {
        self.style.padding = Padding {
            top: padding,
            left: padding,
            bottom: padding,
            right: padding,
        };
        self
    }

    pub fn with_padding_left(mut self, padding: f32) -> Self {
        self.style.padding.left = padding;
        self
    }

    pub fn with_padding_right(mut self, padding: f32) -> Self {
        self.style.padding.right = padding;
        self
    }

    pub fn with_padding_top(mut self, padding: f32) -> Self {
        self.style.padding.top = padding;
        self
    }

    pub fn with_padding_bottom(mut self, padding: f32) -> Self {
        self.style.padding.bottom = padding;
        self
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.style.background_color = Some(color);
        self
    }

    pub fn with_overlay_color(mut self, color: Color) -> Self {
        self.style.overlay_color = Some(color);
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.style.border = border;
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.style.corner_radii.top_left = radius;
        self.style.corner_radii.top_right = radius;
        self.style.corner_radii.bottom_right = radius;
        self.style.corner_radii.bottom_left = radius;
        self
    }

    pub fn with_shadow(mut self, offset: Vector2F, blur: f32, color: Color) -> Self {
        self.style.shadow = Some(Shadow {
            offset,
            blur,
            color,
        });
        self
    }

    pub fn with_cursor(mut self, style: CursorStyle) -> Self {
        self.style.cursor = Some(style);
        self
    }

    fn margin_size(&self) -> Vector2F {
        vec2f(
            self.style.margin.left + self.style.margin.right,
            self.style.margin.top + self.style.margin.bottom,
        )
    }

    fn padding_size(&self) -> Vector2F {
        vec2f(
            self.style.padding.left + self.style.padding.right,
            self.style.padding.top + self.style.padding.bottom,
        )
    }

    fn border_size(&self) -> Vector2F {
        let mut x = 0.0;
        if self.style.border.left {
            x += self.style.border.width;
        }
        if self.style.border.right {
            x += self.style.border.width;
        }

        let mut y = 0.0;
        if self.style.border.top {
            y += self.style.border.width;
        }
        if self.style.border.bottom {
            y += self.style.border.width;
        }

        vec2f(x, y)
    }
}

#[derive(Copy, Clone, Debug, Default, JsonSchema)]
pub struct Border {
    pub color: Color,
    pub width: f32,
    pub overlay: bool,
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl Into<scene::Border> for Border {
    fn into(self) -> scene::Border {
        scene::Border {
            color: self.color,
            left: if self.left { self.width } else { 0.0 },
            right: if self.right { self.width } else { 0.0 },
            top: if self.top { self.width } else { 0.0 },
            bottom: if self.bottom { self.width } else { 0.0 },
        }
    }
}

impl Border {
    pub fn new(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            overlay: false,
            top: false,
            left: false,
            bottom: false,
            right: false,
        }
    }

    pub fn all(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            overlay: false,
            top: true,
            left: true,
            bottom: true,
            right: true,
        }
    }

    pub fn top(width: f32, color: Color) -> Self {
        let mut border = Self::new(width, color);
        border.top = true;
        border
    }

    pub fn left(width: f32, color: Color) -> Self {
        let mut border = Self::new(width, color);
        border.left = true;
        border
    }

    pub fn bottom(width: f32, color: Color) -> Self {
        let mut border = Self::new(width, color);
        border.bottom = true;
        border
    }

    pub fn right(width: f32, color: Color) -> Self {
        let mut border = Self::new(width, color);
        border.right = true;
        border
    }

    pub fn with_sides(mut self, top: bool, left: bool, bottom: bool, right: bool) -> Self {
        self.top = top;
        self.left = left;
        self.bottom = bottom;
        self.right = right;
        self
    }

    pub fn top_width(&self) -> f32 {
        if self.top {
            self.width
        } else {
            0.0
        }
    }

    pub fn left_width(&self) -> f32 {
        if self.left {
            self.width
        } else {
            0.0
        }
    }
}

impl<'de> Deserialize<'de> for Border {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BorderData {
            pub width: f32,
            pub color: Color,
            #[serde(default)]
            pub overlay: bool,
            #[serde(default)]
            pub top: bool,
            #[serde(default)]
            pub right: bool,
            #[serde(default)]
            pub bottom: bool,
            #[serde(default)]
            pub left: bool,
        }

        let data = BorderData::deserialize(deserializer)?;
        let mut border = Border {
            width: data.width,
            color: data.color,
            overlay: data.overlay,
            top: data.top,
            bottom: data.bottom,
            left: data.left,
            right: data.right,
        };
        if !border.top && !border.bottom && !border.left && !border.right {
            border.top = true;
            border.bottom = true;
            border.left = true;
            border.right = true;
        }
        Ok(border)
    }
}

impl ToJson for Border {
    fn to_json(&self) -> serde_json::Value {
        let mut value = json!({});
        if self.top {
            value["top"] = json!(self.width);
        }
        if self.right {
            value["right"] = json!(self.width);
        }
        if self.bottom {
            value["bottom"] = json!(self.width);
        }
        if self.left {
            value["left"] = json!(self.width);
        }
        value
    }
}

impl<V: 'static> Element<V> for Container<V> {
    type LayoutState = ();
    type PaintState = ();

    fn layout(
        &mut self,
        constraint: SizeConstraint,
        view: &mut V,
        cx: &mut ViewContext<V>,
    ) -> (Vector2F, Self::LayoutState) {
        let mut size_buffer = self.margin_size() + self.padding_size();
        if !self.style.border.overlay {
            size_buffer += self.border_size();
        }
        let child_constraint = SizeConstraint {
            min: (constraint.min - size_buffer).max(Vector2F::zero()),
            max: (constraint.max - size_buffer).max(Vector2F::zero()),
        };
        let child_size = self.child.layout(child_constraint, view, cx);
        (child_size + size_buffer, ())
    }

    fn paint(
        &mut self,
        bounds: RectF,
        visible_bounds: RectF,
        _: &mut Self::LayoutState,
        view: &mut V,
        cx: &mut ViewContext<V>,
    ) -> Self::PaintState {
        let quad_bounds = RectF::from_points(
            bounds.origin() + vec2f(self.style.margin.left, self.style.margin.top),
            bounds.lower_right() - vec2f(self.style.margin.right, self.style.margin.bottom),
        );

        if let Some(shadow) = self.style.shadow.as_ref() {
            cx.scene().push_shadow(scene::Shadow {
                bounds: quad_bounds + shadow.offset,
                corner_radii: self.style.corner_radii,
                sigma: shadow.blur,
                color: shadow.color,
            });
        }

        if let Some(hit_bounds) = quad_bounds.intersection(visible_bounds) {
            if let Some(style) = self.style.cursor {
                cx.scene().push_cursor_region(CursorRegion {
                    bounds: hit_bounds,
                    style,
                });
            }
        }

        let child_origin =
            quad_bounds.origin() + vec2f(self.style.padding.left, self.style.padding.top);

        if self.style.border.overlay {
            cx.scene().push_quad(Quad {
                bounds: quad_bounds,
                background: self.style.background_color,
                border: Default::default(),
                corner_radii: self.style.corner_radii.into(),
            });

            self.child.paint(child_origin, visible_bounds, view, cx);

            cx.scene().push_layer(None);
            cx.scene().push_quad(Quad {
                bounds: quad_bounds,
                background: self.style.overlay_color,
                border: self.style.border.into(),
                corner_radii: self.style.corner_radii.into(),
            });
            cx.scene().pop_layer();
        } else {
            cx.scene().push_quad(Quad {
                bounds: quad_bounds,
                background: self.style.background_color,
                border: self.style.border.into(),
                corner_radii: self.style.corner_radii.into(),
            });

            let child_origin = child_origin
                + vec2f(
                    self.style.border.left_width(),
                    self.style.border.top_width(),
                );
            self.child.paint(child_origin, visible_bounds, view, cx);

            if self.style.overlay_color.is_some() {
                cx.scene().push_layer(None);
                cx.scene().push_quad(Quad {
                    bounds: quad_bounds,
                    background: self.style.overlay_color,
                    border: Default::default(),
                    corner_radii: self.style.corner_radii.into(),
                });
                cx.scene().pop_layer();
            }
        }
    }

    fn rect_for_text_range(
        &self,
        range_utf16: Range<usize>,
        _: RectF,
        _: RectF,
        _: &Self::LayoutState,
        _: &Self::PaintState,
        view: &V,
        cx: &ViewContext<V>,
    ) -> Option<RectF> {
        self.child.rect_for_text_range(range_utf16, view, cx)
    }

    fn debug(
        &self,
        bounds: RectF,
        _: &Self::LayoutState,
        _: &Self::PaintState,
        view: &V,
        cx: &ViewContext<V>,
    ) -> serde_json::Value {
        json!({
            "type": "Container",
            "bounds": bounds.to_json(),
            "details": self.style.to_json(),
            "child": self.child.debug(view, cx),
        })
    }
}

impl ToJson for ContainerStyle {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "margin": self.margin.to_json(),
            "padding": self.padding.to_json(),
            "background_color": self.background_color.to_json(),
            "border": self.border.to_json(),
            "corner_radius": self.corner_radii,
            "shadow": self.shadow.to_json(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, JsonSchema)]
pub struct Margin {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl ToJson for Margin {
    fn to_json(&self) -> serde_json::Value {
        let mut value = json!({});
        if self.top > 0. {
            value["top"] = json!(self.top);
        }
        if self.right > 0. {
            value["right"] = json!(self.right);
        }
        if self.bottom > 0. {
            value["bottom"] = json!(self.bottom);
        }
        if self.left > 0. {
            value["left"] = json!(self.left);
        }
        value
    }
}

#[derive(Clone, Copy, Debug, Default, JsonSchema)]
pub struct Padding {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

impl Padding {
    pub fn horizontal(padding: f32) -> Self {
        Self {
            left: padding,
            right: padding,
            ..Default::default()
        }
    }

    pub fn vertical(padding: f32) -> Self {
        Self {
            top: padding,
            bottom: padding,
            ..Default::default()
        }
    }
}

impl<'de> Deserialize<'de> for Padding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let spacing = Spacing::deserialize(deserializer)?;
        Ok(match spacing {
            Spacing::Uniform(size) => Padding {
                top: size,
                left: size,
                bottom: size,
                right: size,
            },
            Spacing::Specific {
                top,
                left,
                bottom,
                right,
            } => Padding {
                top,
                left,
                bottom,
                right,
            },
        })
    }
}

impl<'de> Deserialize<'de> for Margin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let spacing = Spacing::deserialize(deserializer)?;
        Ok(match spacing {
            Spacing::Uniform(size) => Margin {
                top: size,
                left: size,
                bottom: size,
                right: size,
            },
            Spacing::Specific {
                top,
                left,
                bottom,
                right,
            } => Margin {
                top,
                left,
                bottom,
                right,
            },
        })
    }
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Spacing {
    Uniform(f32),
    Specific {
        #[serde(default)]
        top: f32,
        #[serde(default)]
        left: f32,
        #[serde(default)]
        bottom: f32,
        #[serde(default)]
        right: f32,
    },
}

impl Padding {
    pub fn uniform(padding: f32) -> Self {
        Self {
            top: padding,
            left: padding,
            bottom: padding,
            right: padding,
        }
    }
}

impl ToJson for Padding {
    fn to_json(&self) -> serde_json::Value {
        let mut value = json!({});
        if self.top > 0. {
            value["top"] = json!(self.top);
        }
        if self.right > 0. {
            value["right"] = json!(self.right);
        }
        if self.bottom > 0. {
            value["bottom"] = json!(self.bottom);
        }
        if self.left > 0. {
            value["left"] = json!(self.left);
        }
        value
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
pub struct Shadow {
    #[serde(default, deserialize_with = "deserialize_vec2f")]
    #[schemars(with = "Vec::<f32>")]
    offset: Vector2F,
    #[serde(default)]
    blur: f32,
    #[serde(default)]
    color: Color,
}

impl ToJson for Shadow {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "offset": self.offset.to_json(),
            "blur": self.blur,
            "color": self.color.to_json()
        })
    }
}