use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    color::Color,
    math::Vec3,
    pbr::AmbientLight,
    prelude::{
        BuildChildren, Camera, Camera2dBundle, Camera3dBundle, Commands, Component,
        ImageBundle, NodeBundle, Res, TextBundle, Transform,
    },
    scene::SceneBundle,
    text::{Text, TextStyle},
    ui::{
        AlignItems, BorderColor, BorderRadius, Display, FlexDirection, JustifyContent, Style,
        UiImage, UiRect, Val,
    },
    DefaultPlugins,
};
use bevy_mod_opacity::{FadeIn, FadeOut, Opacity, OpacityPlugin, UiOpacity};

#[derive(Debug, Component)]
pub struct OnDelete;

impl Drop for OnDelete {
    fn drop(&mut self) {
        println!("An entity has been deleted!")
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OpacityPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1000.,
        })
        .add_systems(Startup, init)
        .run();
}

pub fn init(mut commands: Commands, assets: Res<AssetServer>) {
    let style = TextStyle {
        font_size: 64.0,
        ..Default::default()
    };
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(4., 4., 4.))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                border: UiRect::all(Val::Px(2.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert((Opacity::INVISIBLE, FadeIn::new(10.)))
        .with_children(|build| {
            build
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Auto,
                        height: Val::Auto,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.0),
                        border: UiRect::all(Val::Px(5.)),
                        padding: UiRect::all(Val::Px(10.)),
                        ..Default::default()
                    },
                    border_color: BorderColor(Color::WHITE),
                    border_radius: BorderRadius::all(Val::Px(20.)),
                    ..Default::default()
                })
                .insert(UiOpacity::Border)
                .with_children(|build| {
                    build.spawn(TextBundle {
                        text: Text::from_section("Made with", style.clone()),
                        ..Default::default()
                    });
                    build.spawn(ImageBundle {
                        image: UiImage {
                            texture: assets.load("heart.png"),
                            ..Default::default()
                        },
                        style: Style {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    build.spawn(TextBundle {
                        text: Text::from_section("using", style.clone()),
                        ..Default::default()
                    });
                    build.spawn(ImageBundle {
                        image: UiImage {
                            texture: assets.load("ferris.png"),
                            ..Default::default()
                        },
                        style: Style {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    build.spawn(TextBundle {
                        text: Text::from_section("and", style.clone()),
                        ..Default::default()
                    });
                    build.spawn(ImageBundle {
                        image: UiImage {
                            texture: assets.load("bevy.png"),
                            ..Default::default()
                        },
                        style: Style {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    build.spawn(TextBundle {
                        text: Text::from_section("!", style),
                        ..Default::default()
                    });
                });
        });
    commands
        .spawn(SceneBundle {
            scene: assets.load("rings1.glb#Scene0"),
            transform: Transform::from_translation(Vec3::new(1., 0., -1.)),
            ..Default::default()
        })
        .insert((Opacity::FULL, FadeOut::new(4.), OnDelete));
    commands
        .spawn(SceneBundle {
            scene: assets.load("rings2.glb#Scene0"),
            transform: Transform::from_translation(Vec3::new(-1., 0., 1.)),
            ..Default::default()
        })
        .insert((
            Opacity::INVISIBLE,
            FadeIn::new(4.),
            OnDelete,
        ));
}
