use bevy::{
    asset::Assets,
    color::Alpha,
    ecs::{query::QueryData, system::SystemParam},
    pbr::{Material, MeshMaterial3d, StandardMaterial},
    prelude::{Component, ResMut},
    sprite::{ColorMaterial, Material2d, MeshMaterial2d, Sprite},
    text::TextColor,
    ui::{BackgroundColor, BorderColor, UiImage},
};

use crate::{OpacityAsset, OpacityQuery};

impl OpacityQuery for &mut Sprite {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_>, _: &mut (), opacity: f32) {
        this.color.set_alpha(opacity);
    }
}

impl OpacityQuery for &mut UiImage {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_>, _: &mut (), opacity: f32) {
        this.color.set_alpha(opacity);
    }
}

impl OpacityQuery for &mut TextColor {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_>, _: &mut (), opacity: f32) {
        this.set_alpha(opacity);
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

impl<T> OpacityQuery for &MeshMaterial2d<T>
where
    T: OpacityAsset + Material2d,
{
    type Cx = ResMut<'static, Assets<T>>;

    fn apply_opacity(
        this: &mut Self::Item<'_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    ) {
        if let Some(mat) = cx.get_mut(this.id()) {
            mat.apply_opacity(opacity);
        }
    }
}

impl<T> OpacityQuery for &MeshMaterial3d<T>
where
    T: OpacityAsset + Material,
{
    type Cx = ResMut<'static, Assets<T>>;

    fn apply_opacity(
        this: &mut Self::Item<'_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    ) {
        if let Some(mat) = cx.get_mut(this.id()) {
            mat.apply_opacity(opacity);
        }
    }
}
