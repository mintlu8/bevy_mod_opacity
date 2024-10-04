//! Magic for using bevy's alpha and f32.

use bevy::color::Alpha;
pub struct BevyAlphaMarker;
pub struct F32Marker;

pub trait SetAlpha<M> {
    fn set_alpha(&mut self, alpha: f32);
}

impl<T: Alpha> SetAlpha<BevyAlphaMarker> for T {
    fn set_alpha(&mut self, alpha: f32) {
        Alpha::set_alpha(self, alpha);
    }
}

impl SetAlpha<F32Marker> for f32 {
    fn set_alpha(&mut self, alpha: f32) {
        *self = alpha;
    }
}

pub fn set_alpha<T: SetAlpha<A>, A>(item: &mut T, alpha: f32) {
    item.set_alpha(alpha);
}
