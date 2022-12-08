use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_reflect::std_traits::ReflectDefault;
use bevy_reflect::Reflect;

/// An identifier for a rendering layer.
pub type Layer = u32;

#[derive(Component, Copy, Clone, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord)]
#[reflect(Component, Default, PartialEq)]
pub struct RenderLayer {
    pub layer: Layer,
}

/// Defaults to containing to layer `0`, the first layer.
impl Default for RenderLayer {
    fn default() -> Self {
        RenderLayer::new(0)
    }
}

impl RenderLayer {
    /// The total number of layers supported.
    pub const TOTAL_LAYERS: usize = Layer::MAX as usize;

    /// Create a new `RenderLayers` belonging to the given layer.
    pub const fn new(layer: Layer) -> Self {
        RenderLayer { layer }
    }
}
