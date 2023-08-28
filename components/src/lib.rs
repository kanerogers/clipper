use std::{
    collections::{HashMap, VecDeque},
    ops::AddAssign,
    sync::Arc,
    time::Instant,
};

use common::{
    glam::{UVec2, Vec2, Vec3, Vec4},
    hecs::{self, Entity},
};
mod beacon;
mod brainwash_state;
mod job;
mod transform;
mod viking;
pub use beacon::Beacon;
pub use brainwash_state::BrainwashState;
pub use job::{Job, JobState};
pub use transform::Transform;
pub use viking::{BrainwashState as VikingState, Viking};

#[derive(Debug, Clone)]
pub struct GLTFAsset {
    pub name: String,
}

impl GLTFAsset {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

pub struct Targeted;
pub struct TargetIndicator(pub hecs::Entity);

/// tag component to indicate that we'd like a collider based on our geometry, please
#[derive(Debug, Clone, Default)]
pub struct Collider {
    pub y_offset: f32,
}

pub struct Parent {
    pub entity: Entity,
    pub offset: Transform,
}

#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

#[derive(Debug, Clone)]
pub struct Dave {
    pub energy: usize,
    pub health: usize,
    pub last_brainwash_time: Instant,
    pub last_energy_drain_time: Instant,
}

impl Dave {
    pub fn new(energy: usize, health: usize) -> Self {
        Self {
            energy,
            health,
            ..Default::default()
        }
    }
}

impl Default for Dave {
    fn default() -> Self {
        Self {
            energy: Default::default(),
            health: Default::default(),
            last_brainwash_time: Instant::now(),
            last_energy_drain_time: Instant::now(),
        }
    }
}

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

    pub const fn work_duration(&self) -> f32 {
        match self {
            Task::Gather => 8.,
            Task::Smelt => 4.,
            Task::MakePaperclips => 5.,
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
    pub primitives: Arc<Vec<Primitive>>,
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

#[derive(Debug, Clone)]
pub struct MaterialOverrides {
    pub base_colour_factor: Vec4,
}
