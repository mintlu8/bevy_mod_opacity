use bevy::{
    color::Alpha,
    ecs::query::QueryData,
    pbr::StandardMaterial,
    prelude::Component,
    sprite::{ColorMaterial, Sprite},
    text::Text,
    ui::{BackgroundColor, BorderColor, UiImage},
};

use crate::{OpacityAsset, OpacityComponent, OpacityQuery};

impl OpacityComponent for Sprite {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        self.color.set_alpha(opacity);
    }
}

impl OpacityComponent for UiImage {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        self.color.set_alpha(opacity);
    }
}

impl OpacityComponent for Text {
    type Cx = ();

    fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
        for section in &mut self.sections {
            section.style.color.set_alpha(opacity);
        }
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
                this.border.0.set_alpha(opacity);
            }
            UiOpacity::Background => {
                this.background.0.set_alpha(opacity);
            }
            UiOpacity::Both => {
                this.border.0.set_alpha(opacity);
                this.background.0.set_alpha(opacity);
            }
        }
    }
}

impl OpacityAsset for ColorMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color.set_alpha(opacity)
    }
}

impl OpacityAsset for StandardMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.base_color.set_alpha(opacity)
    }
}
