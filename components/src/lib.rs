use std::{
    collections::{HashMap, VecDeque},
    ops::AddAssign,
};

use common::{
    glam::{Affine3A, Mat4, Quat, UVec2, Vec2, Vec3, Vec4},
    hecs,
    rapier3d::na,
};
mod beacon;
mod human;
pub use beacon::Beacon;
pub use human::{Human, State as HumanState};

#[derive(Debug, Clone)]
pub struct GLTFAsset {
    pub name: String,
}

impl GLTFAsset {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

/// tag component to indicate that we'd like a collider based on our geometry, please
#[derive(Debug, Clone, Default)]
pub struct Collider {
    pub y_offset: f32,
}

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

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn from_rotation_position(rotation: Quat, position: Vec3) -> Self {
        Self {
            rotation,
            position,
            ..Default::default()
        }
    }
}

impl From<&Transform> for Affine3A {
    fn from(value: &Transform) -> Self {
        Affine3A::from_scale_rotation_translation(value.scale, value.rotation, value.position)
    }
}

impl From<&Transform> for Mat4 {
    fn from(value: &Transform) -> Self {
        Mat4::from_scale_rotation_translation(value.scale, value.rotation, value.position)
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

#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct Dave {}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Resource {
    RawIron,
    Iron,
    Paperclip,
}

impl Resource {
    pub const fn destination(&self) -> ResourceDestination {
        match self {
            Resource::RawIron => ResourceDestination::PlaceOfWork(PlaceType::Forge),
            Resource::Iron => ResourceDestination::PlaceOfWork(PlaceType::Factory),
            Resource::Paperclip => ResourceDestination::Storage,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ResourceDestination {
    PlaceOfWork(PlaceType),
    Storage,
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

#[derive(Debug, Clone, Copy)]
pub enum Task {
    Gather,
    Smelt,
    MakePaperclips,
}

impl Task {
    pub const fn resource(&self) -> Resource {
        match self {
            Task::Gather => Resource::RawIron,
            Task::Smelt => Resource::Iron,
            Task::MakePaperclips => Resource::Paperclip,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaceOfWork {
    pub place_type: PlaceType,
    pub task: Task,
    pub worker_capacity: usize,
    pub workers: VecDeque<hecs::Entity>,
}

impl PlaceOfWork {
    pub fn mine() -> PlaceOfWork {
        PlaceOfWork {
            place_type: PlaceType::Mine,
            task: Task::Gather,
            worker_capacity: 5,
            workers: Default::default(),
        }
    }

    pub fn forge() -> PlaceOfWork {
        PlaceOfWork {
            place_type: PlaceType::Forge,
            task: Task::Smelt,
            worker_capacity: 2,
            workers: Default::default(),
        }
    }

    pub fn factory() -> PlaceOfWork {
        PlaceOfWork {
            place_type: PlaceType::Factory,
            task: Task::MakePaperclips,
            worker_capacity: 1,
            workers: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum PlaceType {
    Mine,
    Forge,
    Factory,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Storage;

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    inner: HashMap<Resource, usize>,
}

impl Inventory {
    pub fn new<H: Into<HashMap<Resource, usize>>>(inner: H) -> Self {
        Self {
            inner: inner.into(),
        }
    }

    pub fn take(&mut self, amount: usize, resource: &Resource) -> Option<usize> {
        println!("Attempting to take {amount} {resource:?} from {self:?}..");
        if let Some(remaining) = self.inner.get_mut(&resource) {
            if *remaining == 0 {
                println!("None left!");
                return None;
            }
            // TODO do this properly
            *remaining = remaining.checked_sub(amount).unwrap_or_default();
            return Some(amount);
        }
        println!("No {resource:?} found!");

        None
    }

    pub fn add(&mut self, resource: Resource, amount: usize) {
        self.inner.entry(resource).or_default().add_assign(amount);
    }

    pub fn amount_of(&self, resource: Resource) -> usize {
        self.inner.get(&resource).copied().unwrap_or_default()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vec4,
    pub normal: Vec4,
    pub uv: Vec2,
}

#[derive(Debug, Clone)]
pub struct GLTFModel {
    pub primitives: Vec<Primitive>,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub base_colour_texture: Option<Texture>,
    pub base_colour_factor: Vec4,
    pub normal_texture: Option<Texture>,
    pub metallic_roughness_ao_texture: Option<Texture>,
    pub emissive_texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_colour_texture: Default::default(),
            base_colour_factor: Vec4::ONE,
            normal_texture: Default::default(),
            metallic_roughness_ao_texture: Default::default(),
            emissive_texture: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    /// x, y
    pub dimensions: UVec2,
    /// data is assumed to be R8G8B8A8
    pub data: Vec<u8>,
}

impl Vertex {
    pub fn new<T: Into<Vec4>, U: Into<Vec2>>(position: T, normal: T, uv: U) -> Self {
        Self {
            position: position.into(),
            normal: normal.into(),
            uv: uv.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}
