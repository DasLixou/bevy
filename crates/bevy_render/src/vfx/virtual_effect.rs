use bevy_reflect::{FromReflect, Reflect, TypeUuid};

use crate::camera::RenderTarget;

#[derive(Reflect, FromReflect, Debug, Clone, TypeUuid)]
#[uuid = "bb59c320-0756-4b90-9276-d2e79b3616a4"]
#[reflect_value]
pub struct VirtualEffect {
    target: RenderTarget,
}
