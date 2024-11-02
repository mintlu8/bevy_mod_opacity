use bevy::{
    color::Alpha,
    ecs::query::QueryData,
    pbr::StandardMaterial,
    prelude::Component,
    sprite::{ColorMaterial, Sprite},
    text::TextColor,
    ui::{BackgroundColor, BorderColor, UiImage},
};

use crate::{OpacityAsset, OpacityComponent, OpacityQuery};

impl OpacityComponent for Sprite {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        Alpha::set_alpha(&mut self.color, opacity);
    }
}

impl OpacityComponent for UiImage {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        Alpha::set_alpha(&mut self.color, opacity);
    }
}

impl OpacityComponent for TextColor {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        Alpha::set_alpha(&mut self.0, opacity);
    }
}

/// Determine whether [`BorderColor`] and [`BackgroundColor`] are controlled by
/// opacity or should stay transparent.
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

    fn apply_opacity(this: &mut Self::Item<'_>, _: &mut (), opacity: f32) {
        match this.ui_color {
            UiOpacity::None => (),
            UiOpacity::Border => {
                Alpha::set_alpha(&mut this.border.0, opacity);
            }
            UiOpacity::Background => {
                Alpha::set_alpha(&mut this.background.0, opacity);
            }
            UiOpacity::Both => {
                Alpha::set_alpha(&mut this.border.0, opacity);
                Alpha::set_alpha(&mut this.background.0, opacity);
            }
        }
    }
}

impl OpacityAsset for ColorMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        Alpha::set_alpha(&mut self.color, opacity)
    }
}

impl OpacityAsset for StandardMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        Alpha::set_alpha(&mut self.base_color, opacity);
    }
}
