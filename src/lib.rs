//! Hierarchical opacity for bevy.
//!
//! # The [`struct@Opacity`] component
//!
//! When `Opacity` is inserted to an entity, the entity and all its descendants
//! will be affected by the opacity value. Unlike bevy components like `Visibility`
//! `Opacity` does not need to be put on every entity in the tree.
//! Entities with no `Opacity` ancestor will not not affected by this crate.
//!
//! # Support for native types
//!
//! We innately support `2d`, `3d` and `ui`, this includes `Sprite`, `TextColor`, `StandardMaterial`,
//! `ColorMaterial`, `Image`, `BackgroundColor` and `ForegroundColor`.
//!
//! Additionally you can implement [`OpacityQuery`] or derive `Opacity` to make your own types
//! and materials work with this crate. Combining `OpacityQuery` with custom `QueryData` can
//! add support for third party types.
//!
//! # [`FadeIn`] and [`FadeOut`]
//!
//! These components adds a quick way to add and remove entities from your scenes smoothly.
//! You should add a [`FadeIn`] during the `spawn` call and use `entity.insert(FadeOut)` instead
//! of `entity.despawn_recursive()`
//!
//! # FAQ
//!
//! * My 3d scene is not fading correctly
//!
//!  Ensure materials are duplicated and unique, since we write to the underlying material directly.
//!  Also make sure `AlphaMode` is set to `Blend` if applicable.

mod alpha;
mod impls;
#[doc(hidden)]
pub use alpha::set_alpha;
#[doc(hidden)]
pub use bevy::asset::{Assets, Handle};
#[doc(hidden)]
pub use bevy::ecs::query::QueryData;

use bevy::ecs::schedule::{ApplyDeferred, IntoScheduleConfigs};
use bevy::ecs::system::Commands;
use bevy::sprite::Material2d;
use bevy::time::{Time, Virtual};
use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::Asset,
    ecs::{
        entity::EntityHashMap,
        system::{StaticSystemParam, SystemParam},
    },
    pbr::{ExtendedMaterial, Material, MaterialExtension, MeshMaterial3d, StandardMaterial},
    prelude::ImageNode,
    prelude::{Children, Component, Entity, Query, Res, ResMut, Resource, SystemSet},
    sprite::{ColorMaterial, MeshMaterial2d, Sprite},
    text::TextColor,
    transform::systems::{propagate_parent_transforms, sync_simple_transforms},
};
pub use impls::UiOpacity;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

#[cfg(feature = "derive")]
pub use bevy_mod_opacity_derive::Opacity;
use impls::UiColorQuery;

/// [`Component`] of opacity of this entity and its children.
#[derive(Debug, Clone, Copy, Component, PartialEq, PartialOrd)]
pub struct Opacity {
    current: f32,
    target: f32,
    speed: f32,
    despawns: bool,
}

impl Opacity {
    /// Opacity `0.0`.
    pub const INVISIBLE: Opacity = Opacity::new(0.);
    /// Opacity `1.0`.
    pub const OPAQUE: Opacity = Opacity::new(1.);

    /// Creates a new opacity value.
    pub const fn new(opacity: f32) -> Opacity {
        Opacity {
            current: opacity,
            target: opacity,
            speed: 0.0,
            despawns: false,
        }
    }

    /// Returns the current opacity value.
    pub const fn get(&self) -> f32 {
        self.current
    }

    /// Returns the target opacity value.
    pub const fn get_target(&self) -> f32 {
        self.target
    }

    /// Set the opacity value and cancels interpolation or fade out.
    pub fn set(&mut self, opacity: f32) {
        *self = Self::new(opacity)
    }

    pub const fn is_opaque(&self) -> bool {
        self.current >= 1.0
    }

    pub const fn is_visible(&self) -> bool {
        self.current > 0.0
    }

    pub const fn is_invisible(&self) -> bool {
        self.current <= 0.0
    }

    /// Set opacity to `0.0` and interpolate to `1.0`.
    pub const fn new_fade_in(time: f32) -> Opacity {
        Opacity {
            current: 0.0,
            target: 1.0,
            speed: 1.0 / time,
            despawns: false,
        }
    }

    /// Interpolate to `1.0`.
    pub const fn and_fade_in(mut self, time: f32) -> Self {
        self.target = 1.0;
        self.speed = 1.0 / time;
        self.despawns = false;
        self
    }

    /// Interpolate opacity to `1.0`.
    pub fn fade_in(&mut self, time: f32) {
        self.target = 1.0;
        self.despawns = false;
        self.speed = 1.0 / time;
    }

    /// Interpolate opacity to `0.0` and despawns the entity when that happens.
    ///
    /// Deletion can be stopped by calling `set` or `fade_in`.
    pub fn fade_out(&mut self, time: f32) {
        self.target = 0.0;
        self.despawns = true;
        self.speed = -1.0 / time;
    }

    /// Interpolate opacity to a specific value.
    pub fn interpolate_to(&mut self, opacity: f32, time: f32) {
        self.target = opacity;
        self.despawns = false;
        self.speed = (opacity - self.current) / time;
    }

    /// Interpolate opacity to a specific value.
    pub fn interpolate_by_speed(&mut self, opacity: f32, time_zero_to_one: f32) {
        self.target = opacity;
        self.despawns = false;
        self.speed = (opacity - self.current).signum() / time_zero_to_one;
    }
}

/// # Why default `1.0`
///
/// It's better to show something by default than hide it implicitly.
impl Default for Opacity {
    fn default() -> Self {
        Self::OPAQUE
    }
}

impl Serialize for Opacity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.target.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Opacity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Opacity::new(f32::deserialize(deserializer)?))
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

/// A [`QueryData`] with an opacity value.
pub trait OpacityQuery: QueryData + Send + Sync {
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

fn interpolate(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut Opacity)>,
) {
    let dt = time.delta_secs();
    for (entity, mut opacity) in &mut query {
        match opacity.speed {
            0.0 => continue,
            s if s > 0.0 => {
                opacity.current += opacity.speed * dt;
                if opacity.current > opacity.target {
                    opacity.current = opacity.target;
                    opacity.speed = 0.0;
                }
            }
            _ => {
                opacity.current += opacity.speed * dt;
                if opacity.current < opacity.target {
                    opacity.current = opacity.target;
                    opacity.speed = 0.0;
                }
            }
        }
        if opacity.despawns && opacity.current <= 0.0 {
            commands.entity(entity).try_despawn();
        }
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
        stack.push((entity, opacity.get()));
        while let Some((entity, opacity)) = stack.pop() {
            map.0.insert(entity, opacity);
            if let Ok(children) = children.get(entity) {
                for entity in children.iter().copied() {
                    let op = query.get(entity).map(|(_, x)| x.get()).unwrap_or(1.);
                    stack.push((entity, opacity * op));
                }
            }
        }
    }
}

/// Add support for writing opacity to a [`QueryData`].
#[derive(Debug)]
pub(crate) struct OpacityQueryPlugin<C: OpacityQuery>(PhantomData<C>);

impl<C: OpacityQuery + 'static> Plugin for OpacityQueryPlugin<C> {
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

pub trait OpacityExtension {
    fn register_opacity<Q: OpacityQuery + 'static>(&mut self) -> &mut Self;
    fn register_opacity_component<C: Component>(&mut self) -> &mut Self
    where
        &'static mut C: OpacityQuery;
    fn register_opacity_material2d<M: Material2d + OpacityAsset>(&mut self) -> &mut Self;
    fn register_opacity_material3d<M: Material + OpacityAsset>(&mut self) -> &mut Self;
}

impl OpacityExtension for App {
    fn register_opacity<Q: OpacityQuery + 'static>(&mut self) -> &mut Self {
        self.add_plugins(OpacityQueryPlugin::<Q>(PhantomData));
        self
    }

    fn register_opacity_component<C: Component>(&mut self) -> &mut Self
    where
        &'static mut C: OpacityQuery,
    {
        self.add_plugins(OpacityQueryPlugin::<&mut C>(PhantomData));
        self
    }

    fn register_opacity_material2d<M: Material2d + OpacityAsset>(&mut self) -> &mut Self {
        self.add_plugins(OpacityQueryPlugin::<&MeshMaterial2d<M>>(PhantomData));
        self
    }

    fn register_opacity_material3d<M: Material + OpacityAsset>(&mut self) -> &mut Self {
        self.add_plugins(OpacityQueryPlugin::<&MeshMaterial3d<M>>(PhantomData));
        self
    }
}

impl Plugin for OpacityPlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::view::VisibilitySystems::*;
        use OpacitySet::*;
        app.init_resource::<OpacityMap>();
        app.configure_sets(
            PostUpdate,
            (Fading, PostFade, Calculate, Apply)
                .chain()
                .after(propagate_parent_transforms)
                .after(sync_simple_transforms)
                .before(CheckVisibility)
                .before(UpdateFrusta),
        );
        app.add_systems(PostUpdate, interpolate.in_set(Fading));
        app.add_systems(PostUpdate, ApplyDeferred.in_set(PostFade));
        app.add_systems(PostUpdate, calculate_opacity.in_set(Calculate));
        app.register_opacity_component::<Sprite>();
        app.register_opacity_component::<TextColor>();
        app.register_opacity_component::<ImageNode>();
        app.register_opacity_material2d::<ColorMaterial>();
        app.register_opacity_material3d::<StandardMaterial>();
        app.register_opacity::<UiColorQuery>();
    }
}
