use common::glam::{Affine3A, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            scale: Vec3::ONE,
            rotation: Default::default(),
        }
    }
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn from_rotation_translation(rotation: Quat, translation: Vec3) -> Self {
        Self {
            rotation,
            translation,
            ..Default::default()
        }
    }
}

impl From<&Transform> for Affine3A {
    fn from(value: &Transform) -> Self {
        Affine3A::from_scale_rotation_translation(value.scale, value.rotation, value.translation)
    }
}
