use common::glam::{Affine3A, Quat, Vec3};
use common::rapier3d::na;
mod human;
pub use human::{Human, State as HumanState};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: Vec3::ONE,
            rotation: Default::default(),
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            scale,
            rotation,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            position: translation,
            ..Default::default()
        }
    }

    pub fn from_rotation_translation(rotation: Quat, translation: Vec3) -> Self {
        Self {
            rotation,
            position: translation,
            ..Default::default()
        }
    }
}

impl From<&Transform> for Affine3A {
    fn from(value: &Transform) -> Self {
        Affine3A::from_scale_rotation_translation(value.scale, value.rotation, value.position)
    }
}

impl From<&Transform> for na::Isometry3<f32> {
    fn from(value: &Transform) -> Self {
        na::Isometry::from_parts(
            value.position.to_array().into(),
            na::UnitQuaternion::from_quaternion(na::Quaternion::from_parts(
                value.rotation.w,
                value.rotation.xyz().to_array().into(),
            )),
        )
    }
}

// impl From<&na::Isometry3<f32>> for Transform {
//     fn from(value: &na::Isometry3<f32>) -> Self {
//         let t = value.translation;
//         let r = value.rotation.quaternion();
//         Transform { position: [t.x, t.y, t.z], scale: Vec3::O, rotation: () }
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct Dave {}

#[derive(Debug, Clone, Default)]
pub struct Resource {
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, Default)]
pub enum ResourceType {
    #[default]
    Wood,
}

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub name: String,
}

impl Info {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Selected;
