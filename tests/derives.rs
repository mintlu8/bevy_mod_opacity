use bevy::{
    asset::{Asset, Assets},
    color::Srgba,
    ecs::system::RunSystemOnce,
    pbr::MaterialExtension,
    pbr::StandardMaterial,
    prelude::{ResMut, World},
    reflect::TypePath,
    render::render_resource::AsBindGroup,
};
use bevy_mod_opacity::{Opacity, OpacityComponent};

#[derive(Debug, Opacity)]
pub struct MyColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[opacity]
    pub a: f32,
}

#[derive(Debug, TypePath, Asset, Opacity)]
#[opacity(asset)]
pub struct MyColorMaterial {
    #[opacity]
    pub color: Srgba,
}

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
    let mut color = MyColor {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    color.apply_opacity(&mut (), 1.);

    assert_eq!(color.a, 1.);

    let mut world = World::new();

    let mut assets = Assets::<MyColorMaterial>::default();
    let mut id = assets.add(MyColorMaterial {
        color: Srgba::WHITE,
    });
    world.insert_resource(assets);

    world.run_system_once(move |mut res: ResMut<Assets<MyColorMaterial>>| {
        id.apply_opacity(&mut res, 0.5);
    });

    assert_eq!(
        world
            .resource::<Assets<MyColorMaterial>>()
            .iter()
            .next()
            .map(|(_, x)| x.color.alpha),
        Some(0.5)
    );
}
