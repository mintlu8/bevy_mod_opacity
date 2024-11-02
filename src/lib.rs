//! Hierarchical opacity for bevy.
//!
//! # The [`struct@Opacity`] component
//!
//! When `Opacity` is put on an entity, all its descendants will be affected by the opacity value.
//! Entity with no `Opacity` ancestor is not affected by this crate.
//!
//! # Support for native types
//!
//! We innately support `2d`, `3d` and `ui`, this includes `Sprite`, `Text`, `Handle<StandardMaterial>`,
//! `Handle<ColorMaterial>`, `Image`, `BackgroundColor` and `ForegroundColor`.
//!
//! Additionally you can implement [`OpacityComponent`] or derive `Opacity` to make your own types
//! ang materials work with this crate, more complicated behaviors can be achieved through [`OpacityQuery`] by
//! deriving [`QueryData`] and implementing [`OpacityQuery`] manually. This is the preferred way
//! to add support for third party types.
//!
//! # [`FadeIn`] and [`FadeOut`]
//!
//! These components adds a quick way to add and remove items from your scenes smoothly
//! as a complement to `insert` and an alternative to `remove`.
//!
//! # Pitfalls
//!
//! Spawned scenes might share material from the same handle,
//! cloning materials is required for this crate to function properly.

mod alpha;
mod fading;
mod impls;
#[doc(hidden)]
pub use alpha::set_alpha;
#[doc(hidden)]
pub use bevy::asset::{Assets, Handle};
use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::Asset,
    ecs::{
        entity::EntityHashMap,
        query::QueryData,
        system::{StaticSystemParam, SystemParam},
    },
    pbr::{ExtendedMaterial, Material, MaterialExtension},
    prelude::{
        Children, Component, Entity, IntoSystemConfigs, IntoSystemSetConfigs, Query, Res, ResMut,
        Resource, SystemSet,
    },
    sprite::Sprite,
    text::TextColor,
    transform::systems::{propagate_transforms, sync_simple_transforms},
    ui::UiImage,
};
use fading::{fade_in, fade_out};
pub use fading::{FadeIn, FadeOut};
pub use impls::UiOpacity;
use std::marker::PhantomData;

#[cfg(feature = "derive")]
pub use bevy_mod_opacity_derive::Opacity;
use impls::UiColorQuery;

/// [`Component`] of opacity of this entity and its children.
#[derive(Debug, Clone, Copy, Component, PartialEq, PartialOrd)]
pub struct Opacity(pub f32);

impl Opacity {
    pub const INVISIBLE: Opacity = Opacity(0.);
    pub const FULL: Opacity = Opacity(1.);
}

/// # Why default `1.0`
///
/// It's better to show something by default than hide it implicitly.
impl Default for Opacity {
    fn default() -> Self {
        Self(1.0)
    }
}

/// A map of entity to opacity, if not present, the entity does not have an opacity root node.
/// This means the entity is out of the scope of this crate and should not be handled.
#[derive(Debug, Resource, Default)]
pub struct OpacityMap(EntityHashMap<f32>);

/// [`SystemSet`] of opacity,
/// runs in [`PostUpdate`] between transform propagation and visibility calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum OpacitySet {
    Fading,
    PostFade,
    Calculate,
    Apply,
}

/// A [`Component`] with an opacity value.
pub trait OpacityComponent {
    type Cx: SystemParam;

    fn apply_opacity(&mut self, cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>, opacity: f32);
}

/// A [`QueryData`] with an opacity value.
pub trait OpacityQuery: QueryData + Send + Sync + 'static {
    type Cx: SystemParam;

    fn apply_opacity(
        this: &mut Self::Item<'_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    );
}

/// An [`Asset`] with an opacity value.
pub trait OpacityAsset: Asset {
    fn apply_opacity(&mut self, opacity: f32);
}

impl<T> OpacityComponent for Handle<T>
where
    T: OpacityAsset,
{
    type Cx = ResMut<'static, Assets<T>>;

    fn apply_opacity(&mut self, cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>, opacity: f32) {
        if let Some(asset) = cx.get_mut(self.id()) {
            OpacityAsset::apply_opacity(asset, opacity);
        }
    }
}

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

fn calculate_opacity(
    mut map: ResMut<OpacityMap>,
    query: Query<(Entity, &Opacity)>,
    children: Query<&Children>,
) {
    map.0.clear();
    let mut stack = Vec::new();
    for (entity, opacity) in &query {
        if map.0.contains_key(&entity) {
            continue;
        }
        stack.push((entity, opacity.0));
        while let Some((entity, opacity)) = stack.pop() {
            map.0.insert(entity, opacity);
            if let Ok(children) = children.get(entity) {
                for entity in children.iter().copied() {
                    let op = query.get(entity).map(|(_, x)| x.0).unwrap_or(1.);
                    stack.push((entity, opacity * op));
                }
            }
        }
    }
}

/// Add support for writing opacity to a [`Component`].
#[derive(Debug)]
pub struct OpacityComponentPlugin<C: OpacityComponent>(PhantomData<C>);

impl<C: OpacityComponent> OpacityComponentPlugin<C> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<C: OpacityComponent> Default for OpacityComponentPlugin<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: OpacityComponent + Component> Plugin for OpacityComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, apply_opacity::<C>.in_set(OpacitySet::Apply));
    }
}

fn apply_opacity<C: OpacityComponent + Component>(
    map: Res<OpacityMap>,
    cx: StaticSystemParam<C::Cx>,
    mut query: Query<(Entity, &mut C)>,
) {
    let mut cx = cx.into_inner();
    for (entity, mut component) in &mut query {
        if let Some(opacity) = map.0.get(&entity) {
            component.apply_opacity(&mut cx, *opacity)
        }
    }
}
/// Add support for writing opacity to a [`QueryData`].
#[derive(Debug)]
pub struct OpacityQueryPlugin<C: OpacityQuery>(PhantomData<C>);

impl<C: OpacityQuery> OpacityQueryPlugin<C> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<C: OpacityQuery> Default for OpacityQueryPlugin<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: OpacityQuery> Plugin for OpacityQueryPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            apply_opacity_query::<C>.in_set(OpacitySet::Apply),
        );
    }
}

fn apply_opacity_query<Q: OpacityQuery>(
    map: Res<OpacityMap>,
    cx: StaticSystemParam<Q::Cx>,
    mut query: Query<(Entity, Q)>,
) {
    let mut cx = cx.into_inner();
    for (entity, mut component) in &mut query {
        if let Some(opacity) = map.0.get(&entity) {
            Q::apply_opacity(&mut component, &mut cx, *opacity);
        }
    }
}

/// Plugin for [`bevy_mod_opacity`](crate) that adds support for basic bevy types.
pub struct OpacityPlugin;

impl Plugin for OpacityPlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::view::VisibilitySystems::*;
        use OpacitySet::*;
        app.init_resource::<OpacityMap>();
        app.configure_sets(
            PostUpdate,
            (Fading, PostFade, Calculate, Apply)
                .chain()
                .after(propagate_transforms)
                .after(sync_simple_transforms)
                .before(CheckVisibility)
                .before(UpdateFrusta),
        );
        app.add_systems(PostUpdate, (fade_in, fade_out).in_set(Fading));
        app.add_systems(PostUpdate, calculate_opacity.in_set(Calculate));
        app.add_plugins(OpacityComponentPlugin::<Sprite>::new());
        app.add_plugins(OpacityComponentPlugin::<TextColor>::new());
        app.add_plugins(OpacityComponentPlugin::<UiImage>::new());
        //app.add_plugins(OpacityComponentPlugin::<Handle<ColorMaterial>>::new());
        //app.add_plugins(OpacityComponentPlugin::<Handle<StandardMaterial>>::new());
        app.add_plugins(OpacityQueryPlugin::<UiColorQuery>::new());
    }
}
