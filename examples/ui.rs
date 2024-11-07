use bevy::{
    app::{App, Startup},
    asset::AssetServer,
    color::Color,
    math::Vec3,
    pbr::AmbientLight,
    prelude::{
        BuildChildren, Camera, Camera2d, Camera3d, ChildBuild, Commands, Component, Res, Text,
        Transform,
    },
    scene::SceneRoot,
    text::TextFont,
    ui::{
        AlignItems, BorderColor, BorderRadius, Display, FlexDirection, JustifyContent, Node,
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
    let style = TextFont {
        font_size: 64.0,
        ..Default::default()
    };
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..Default::default()
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(4., 4., 4.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(5.0),
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        })
        .insert((Opacity::INVISIBLE, FadeIn::new(10.)))
        .with_children(|build| {
            build
                .spawn((
                    Node {
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
                    BorderColor(Color::WHITE),
                    BorderRadius::all(Val::Px(20.)),
                    UiOpacity::Border,
                ))
                .with_children(|build| {
                    build.spawn((Text::new("Made with"), style.clone()));
                    build.spawn((
                        UiImage::new(assets.load("heart.png")),
                        Node {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                    ));
                    build.spawn((Text::new("using"), style.clone()));
                    build.spawn((
                        UiImage::new(assets.load("ferris.png")),
                        Node {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                    ));
                    build.spawn((Text::new("and"), style.clone()));
                    build.spawn((
                        UiImage::new(assets.load("bevy.png")),
                        Node {
                            width: Val::Auto,
                            padding: UiRect::horizontal(Val::Px(5.0)),
                            height: Val::Px(64.0),
                            ..Default::default()
                        },
                    ));
                    build.spawn((Text::new("!"), style.clone()));
                });
        });
    commands.spawn((
        SceneRoot(assets.load("rings1.glb#Scene0")),
        Transform::from_translation(Vec3::new(1., 0., -1.)),
        FadeOut::new(4.),
        OnDelete,
    ));
    commands.spawn((
        SceneRoot(assets.load("rings2.glb#Scene0")),
        Transform::from_translation(Vec3::new(-1., 0., 1.)),
        FadeIn::new(4.),
        OnDelete,
    ));
}
