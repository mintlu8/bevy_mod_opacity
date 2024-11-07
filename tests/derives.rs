use bevy::{
    app::App,
    asset::Asset,
    color::Srgba,
    pbr::{ExtendedMaterial, Material, MaterialExtension, StandardMaterial},
    prelude::Component,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
};
use bevy_mod_opacity::{Opacity, OpacityExtension, OpacityPlugin};

#[derive(Debug, Component, Opacity)]
pub struct MyColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[opacity]
    pub a: f32,
}

#[derive(Debug, Clone, TypePath, Asset, Opacity, AsBindGroup)]
#[opacity(asset)]
pub struct MyColorMaterial {
    #[opacity]
    pub color: Srgba,
}

impl Material for MyColorMaterial {}

#[derive(Debug, Clone, AsBindGroup, TypePath, Asset, Opacity)]
#[opacity(extends = StandardMaterial)]
pub struct MyColorMaterialExt {
    #[opacity]
    pub color: Srgba,
}

impl MaterialExtension for MyColorMaterialExt {}

#[derive(Debug, Clone, AsBindGroup, TypePath, Asset, Opacity)]
#[opacity(masks = StandardMaterial)]
pub struct MyColorMaterialExtMask {
    #[opacity]
    pub color: Srgba,
}

impl MaterialExtension for MyColorMaterialExtMask {}

#[test]
fn test() {
    let _app = App::new()
        .add_plugins(OpacityPlugin)
        .register_opacity_component::<MyColor>()
        .register_opacity_material3d::<MyColorMaterial>()
        .register_opacity_material3d::<ExtendedMaterial<StandardMaterial, MyColorMaterialExt>>()
        .register_opacity_material3d::<ExtendedMaterial<StandardMaterial, MyColorMaterialExtMask>>(
        );
}
