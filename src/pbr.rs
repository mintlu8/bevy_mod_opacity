use bevy::{
    ecs::system::SystemParam,
    pbr::{
        decal::ForwardDecalMaterialExt, wireframe::WireframeMaterial, ExtendedMaterial, Material,
        MaterialExtension, MeshMaterial3d, StandardMaterial,
    },
    prelude::*,
};

use crate::{OpacityAsset, OpacityExtension, OpacityQuery};

/// A [`MaterialExtension`] with an opacity value.
pub trait OpacityMaterialExtension<A> {
    fn apply_opacity(a: &mut A, b: &mut Self, opacity: f32);
}

impl<A: Material, T: MaterialExtension> OpacityAsset for ExtendedMaterial<A, T>
where
    T: OpacityMaterialExtension<A>,
{
    fn apply_opacity(&mut self, opacity: f32) {
        OpacityMaterialExtension::apply_opacity(&mut self.base, &mut self.extension, opacity);
    }
}

impl<T: OpacityAsset> OpacityMaterialExtension<T> for ForwardDecalMaterialExt {
    fn apply_opacity(a: &mut T, _: &mut Self, opacity: f32) {
        a.apply_opacity(opacity);
    }
}

impl OpacityAsset for StandardMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.base_color.set_alpha(opacity)
    }
}

impl OpacityAsset for WireframeMaterial {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color.set_alpha(opacity)
    }
}

impl<T> OpacityQuery for &MeshMaterial3d<T>
where
    T: OpacityAsset + Material,
{
    type Cx = ResMut<'static, Assets<T>>;

    fn apply_opacity(
        this: &mut Self::Item<'_, '_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    ) {
        if let Some(mat) = cx.get_mut(this.id()) {
            mat.apply_opacity(opacity);
        }
    }
}

pub fn opacity_plugin_3d(app: &mut App) {
    app.register_opacity_material3d::<bevy::pbr::StandardMaterial>();
}
