#![doc = include_str!("../README.md")]
#[doc(hidden)]
pub use bevy::asset::{Assets, Handle};
use bevy::camera::visibility::VisibilitySystems;
#[doc(hidden)]
pub use bevy::color::Alpha;
use bevy::ecs::query::IterQueryData;
#[doc(hidden)]
pub use bevy::ecs::query::QueryData;

use bevy::ecs::schedule::{ApplyDeferred, IntoScheduleConfigs};
use bevy::ecs::system::Commands;
use bevy::time::{Time, Virtual};
use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::Asset,
    ecs::{
        entity::EntityHashMap,
        system::{StaticSystemParam, SystemParam},
    },
    prelude::{Children, Component, Entity, Query, Res, ResMut, Resource, SystemSet},
    transform::systems::{propagate_parent_transforms, sync_simple_transforms},
};
use std::marker::PhantomData;
use std::mem;

#[cfg(feature = "derive")]
pub use bevy_mod_opacity_derive::Opacity;

#[cfg(feature = "reflect")]
use bevy::prelude::{ReflectComponent, ReflectDefault};

#[cfg(feature = "3d")]
mod pbr;
#[cfg(feature = "2d")]
mod sprite;
#[cfg(feature = "ui")]
mod ui;
#[cfg(feature = "3d")]
pub use pbr::OpacityMaterialExtension;
#[cfg(feature = "ui")]
pub use ui::UiOpacity;

/// [`Component`] of opacity of this entity and its children.
#[derive(Debug, Clone, Copy, Component, PartialEq, PartialOrd)]
#[cfg_attr(feature = "reflect", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component, Default))]
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

    /// Returns true if opacity is greater than or equal to `1.0`.
    pub const fn is_opaque(&self) -> bool {
        self.current >= 1.0
    }

    /// Returns true if opacity is greater than to `0.0`.
    pub const fn is_visible(&self) -> bool {
        self.current > 0.0
    }

    /// Returns true if opacity is less than or equal to `0.0`.
    pub const fn is_invisible(&self) -> bool {
        self.current <= 0.0
    }

    /// Returns true if is despawning, only when `fade_out` was called but not completed.
    pub const fn is_despawning(&self) -> bool {
        self.despawns
    }

    /// Set opacity to `0.0` and interpolate to `1.0`.
    pub const fn new_fade_in(seconds: f32) -> Opacity {
        Opacity {
            current: 0.0,
            target: 1.0,
            speed: 1.0 / seconds,
            despawns: false,
        }
    }

    /// Interpolate to `1.0`.
    pub const fn and_fade_in(mut self, seconds: f32) -> Self {
        self.target = 1.0;
        self.speed = 1.0 / seconds;
        self.despawns = false;
        self
    }

    /// Interpolate opacity to `1.0`.
    pub fn fade_in(&mut self, seconds: f32) {
        self.target = 1.0;
        self.despawns = false;
        self.speed = 1.0 / seconds;
    }

    /// Interpolate opacity to `0.0` and despawns the entity when that happens.
    ///
    /// Deletion can be stopped by calling `set`, `fade_in` or `interpolate_to` before fade out completed.
    /// If deletion is not desired, call `interpolate_to` with opacity `0.0` instead.
    pub fn fade_out(&mut self, seconds: f32) {
        self.target = 0.0;
        self.despawns = true;
        self.speed = -1.0 / seconds;
    }

    /// Interpolate opacity to a specific value.
    pub fn interpolate_to(&mut self, opacity: f32, seconds: f32) {
        self.target = opacity;
        self.despawns = false;
        self.speed = (opacity - self.current) / seconds;
    }

    /// Interpolate opacity to a specific value.
    pub fn interpolate_by_speed(&mut self, opacity: f32, seconds_zero_to_one: f32) {
        self.target = opacity;
        self.despawns = false;
        self.speed = (opacity - self.current).signum() / seconds_zero_to_one;
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

#[cfg(feature = "serde")]
const _: () = {
    use ::serde::{Deserialize, Deserializer, Serialize, Serializer};

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
};

/// A map of entity to opacity, if not present, the entity does not have an opacity root node.
/// This means the entity is out of the scope of this crate and should not be handled.
#[derive(Debug, Resource, Default)]
pub struct OpacityMap(EntityHashMap<OpacityEntry>);

#[derive(Debug, Clone, Copy)]
pub struct OpacityEntry {
    pub opacity: f32,
    pub changed: bool,
}

/// [`SystemSet`] of opacity,
/// runs in [`PostUpdate`] between transform propagation and visibility calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub enum OpacitySet {
    /// Modify opacity via fading.
    Fading,
    /// Utilize changed opacity value.
    PostFade,
    /// Calculate accumulated opacity value.
    Calculate,
    /// Apply calculated opacity value.
    Apply,
}

/// A [`QueryData`] with an opacity value.
pub trait OpacityQuery: IterQueryData + Send + Sync {
    type Cx: SystemParam;

    fn apply_opacity(
        this: &mut Self::Item<'_, '_>,
        cx: &mut <Self::Cx as SystemParam>::Item<'_, '_>,
        opacity: f32,
    );
}

/// An [`Asset`] with an opacity value.
pub trait OpacityAsset: Asset {
    fn apply_opacity(&mut self, opacity: f32);
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
    let prev = mem::take(&mut *map);
    let mut stack = Vec::new();
    for (entity, opacity) in &query {
        if map.0.contains_key(&entity) {
            continue;
        }
        stack.push((entity, opacity.get()));
        while let Some((entity, opacity)) = stack.pop() {
            let changed = match prev.0.get(&entity) {
                Some(entry) => entry.opacity != opacity,
                None => true,
            };
            map.0.insert(entity, OpacityEntry { opacity, changed });
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
            if opacity.changed {
                Q::apply_opacity(&mut component, &mut cx, opacity.opacity);
            }
        }
    }
}

/// Plugin for [`bevy_mod_opacity`](crate) that adds support for basic bevy types.
pub struct OpacityPlugin;

/// Extensions for [`App`].
pub trait OpacityExtension {
    fn register_opacity<Q: OpacityQuery + 'static>(&mut self) -> &mut Self;
    fn register_opacity_component<C: Component>(&mut self) -> &mut Self
    where
        &'static mut C: OpacityQuery;
    #[cfg(feature = "2d")]
    fn register_opacity_material2d<M: bevy::sprite_render::Material2d + OpacityAsset>(
        &mut self,
    ) -> &mut Self;
    #[cfg(feature = "3d")]
    fn register_opacity_material3d<M: bevy::pbr::Material + OpacityAsset>(&mut self) -> &mut Self;
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

    #[cfg(feature = "2d")]
    fn register_opacity_material2d<M: bevy::sprite_render::Material2d + OpacityAsset>(
        &mut self,
    ) -> &mut Self {
        self.add_plugins(
            OpacityQueryPlugin::<&bevy::sprite_render::MeshMaterial2d<M>>(PhantomData),
        );
        self
    }

    #[cfg(feature = "3d")]
    fn register_opacity_material3d<M: bevy::pbr::Material + OpacityAsset>(&mut self) -> &mut Self {
        self.add_plugins(OpacityQueryPlugin::<&bevy::pbr::MeshMaterial3d<M>>(
            PhantomData,
        ));
        self
    }
}

#[cfg(any(feature = "2d", feature = "ui"))]
impl OpacityQuery for &mut bevy::text::TextColor {
    type Cx = ();

    fn apply_opacity(this: &mut Self::Item<'_, '_>, _: &mut (), opacity: f32) {
        this.set_alpha(opacity);
    }
}

impl Plugin for OpacityPlugin {
    fn build(&self, app: &mut App) {
        use OpacitySet::*;
        app.init_resource::<OpacityMap>();
        app.configure_sets(
            PostUpdate,
            (Fading, PostFade, Calculate, Apply)
                .chain()
                .after(propagate_parent_transforms)
                .after(sync_simple_transforms)
                .before(VisibilitySystems::CheckVisibility)
                .before(VisibilitySystems::UpdateFrusta),
        );
        app.add_systems(PostUpdate, interpolate.in_set(Fading));
        app.add_systems(PostUpdate, ApplyDeferred.in_set(PostFade));
        app.add_systems(PostUpdate, calculate_opacity.in_set(Calculate));
        #[cfg(any(feature = "2d", feature = "ui"))]
        app.register_opacity_component::<bevy::text::TextColor>();
        #[cfg(feature = "2d")]
        sprite::opacity_plugin_2d(app);
        #[cfg(feature = "3d")]
        pbr::opacity_plugin_3d(app);
        #[cfg(feature = "ui")]
        ui::opacity_plugin_ui(app);
    }
}
