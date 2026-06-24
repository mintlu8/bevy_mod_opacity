use bevy::{
    app::App,
    asset::Assets,
    color::Alpha,
    ecs::system::{ResMut, SystemParam},
    sprite::Sprite,
    sprite_render::{ColorMaterial, Material2d, MeshMaterial2d, Wireframe2dMaterial},
};

use crate::{OpacityAsset, OpacityExtension, OpacityQuery};

impl OpacityQuery for &mut Sprite {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_, '_>, _: &mut (), opacity: f32) {
        this.color.set_alpha(opacity);
    }
}

impl OpacityAsset for ColorMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color.set_alpha(opacity)
    }
}

impl OpacityAsset for Wireframe2dMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color.set_alpha(opacity)
    }
}

impl<T> OpacityQuery for &MeshMaterial2d<T>
where
    T: OpacityAsset + Material2d,
{
    type Cx = ResMut<'static, Assets<T>>;

    fn apply_opacity(
        this: &mut Self::Item<'_, '_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    ) {
        if let Some(mut mat) = cx.get_mut(this.id()) {
            mat.apply_opacity(opacity);
        }
    }
}

pub fn opacity_plugin_2d(app: &mut App) {
    app.register_opacity_component::<Sprite>();
    app.register_opacity_material2d::<ColorMaterial>();
}
