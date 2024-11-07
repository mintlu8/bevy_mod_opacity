use bevy::{
    asset::Asset,
    pbr::{ExtendedMaterial, Material, MaterialExtension, StandardMaterial},
    prelude::{AlphaMode, Commands, Component, DespawnRecursiveExt, Entity, Query, Res},
    time::{Time, Virtual},
};

use crate::Opacity;

/// When inserted, gradually increase opacity to `1.0` within the given time.
///
/// This component is removed afterwards and opacity is
/// guaranteed to be equal to `1.0` after this is removed.
#[derive(Debug, Clone, Copy, Component)]
#[require(Opacity(||Opacity::INVISIBLE))]
pub struct FadeIn {
    pub(crate) current: f32,
    pub(crate) time: f32,
    pub(crate) curve: Option<fn(f32) -> f32>,
}

/// When inserted, gradually decrease opacity to `0.0` within the given time.
///
/// This entity and all its children will be removed afterwards.
#[derive(Debug, Clone, Copy, Component)]
#[require(Opacity(||Opacity::FULL))]
pub struct FadeOut {
    pub(crate) current: f32,
    pub(crate) time: f32,
    pub(crate) curve: Option<fn(f32) -> f32>,
}

impl FadeIn {
    pub fn new(time: f32) -> Self {
        FadeIn {
            current: 0.,
            time,
            curve: None,
        }
    }

    /// Set a curve for fading.
    ///
    /// Curve maps a value in `0..1` to a value in `0..1`,
    /// for example `|x| x`.
    pub fn with_curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = Some(curve);
        self
    }
}

impl FadeOut {
    pub fn new(time: f32) -> Self {
        FadeOut {
            current: 0.,
            time,
            curve: None,
        }
    }

    /// Set a curve for fading.
    ///
    /// Curve maps a value in `0..1` to a value in `0..1`,
    /// for example `|x| x`, does not need to be reversed.
    pub fn with_curve(mut self, curve: fn(f32) -> f32) -> Self {
        self.curve = Some(curve);
        self
    }
}

pub fn fade_in(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut FadeIn, &mut Opacity)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fade_in, mut opacity) in &mut query {
        // Without a curve we can make this work with external modification.
        if let Some(curve) = fade_in.curve {
            fade_in.current += dt;
            opacity.0 = curve(fade_in.current / fade_in.time);
        } else {
            let offset = dt / fade_in.time;
            opacity.0 += offset;
        }
        if opacity.0 > 1. {
            opacity.0 = 1.;
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

pub fn fade_out(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut FadeOut, &mut Opacity)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fade_out, mut opacity) in &mut query {
        // Without a curve we can make this work with external modification.
        if let Some(curve) = fade_out.curve {
            fade_out.current += dt;
            opacity.0 = 1.0 - curve(fade_out.current / fade_out.time);
        } else {
            let offset = dt / fade_out.time;
            opacity.0 -= offset;
        }
        if opacity.0 <= 0. {
            opacity.0 = 0.;
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub trait AlphaModeMaterial: Asset {
    fn set_alpha_mode(&mut self, alpha_mode: AlphaMode);
}

impl AlphaModeMaterial for StandardMaterial {
    fn set_alpha_mode(&mut self, alpha_mode: AlphaMode) {
        self.alpha_mode = alpha_mode;
    }
}

impl<A: Material, B: MaterialExtension> AlphaModeMaterial for ExtendedMaterial<A, B>
where
    A: AlphaModeMaterial,
{
    fn set_alpha_mode(&mut self, alpha_mode: AlphaMode) {
        self.base.set_alpha_mode(alpha_mode);
    }
}
