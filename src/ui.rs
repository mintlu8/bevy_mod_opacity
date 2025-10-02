use crate::{OpacityExtension, OpacityQuery};
use bevy::ui::{BackgroundColor, BorderColor};
use bevy::{
    app::App,
    color::Alpha,
    ecs::query::QueryData,
    prelude::{Component, ImageNode},
};

impl OpacityQuery for &mut ImageNode {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_, '_>, _: &mut (), opacity: f32) {
        this.color.set_alpha(opacity);
    }
}

/// Determine whether [`BorderColor`] and [`BackgroundColor`] are controlled by
/// opacity or should stay transparent.
///
/// Items without this component are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum UiOpacity {
    /// Both should stay transparent
    #[default]
    None,
    /// Opacity controls border color.
    Border,
    /// Opacity controls background color.
    Background,
    /// Opacity controls border and background color.
    Both,
}

#[derive(Debug, QueryData)]
#[query_data(mutable)]
pub struct UiColorQuery {
    pub ui_color: &'static UiOpacity,
    pub background: &'static mut BackgroundColor,
    pub border: &'static mut BorderColor,
}

impl OpacityQuery for UiColorQuery {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_, '_>, _: &mut (), opacity: f32) {
        match this.ui_color {
            UiOpacity::None => (),
            UiOpacity::Border => {
                this.border.left.set_alpha(opacity);
                this.border.right.set_alpha(opacity);
                this.border.top.set_alpha(opacity);
                this.border.bottom.set_alpha(opacity);
            }
            UiOpacity::Background => {
                this.background.0.set_alpha(opacity);
            }
            UiOpacity::Both => {
                this.border.left.set_alpha(opacity);
                this.border.right.set_alpha(opacity);
                this.border.top.set_alpha(opacity);
                this.border.bottom.set_alpha(opacity);
                this.background.0.set_alpha(opacity);
            }
        }
    }
}

pub fn opacity_plugin_ui(app: &mut App) {
    app.register_opacity_component::<ImageNode>();
    app.register_opacity::<UiColorQuery>();
}
