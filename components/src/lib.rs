use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    ops::{AddAssign, Sub},
    sync::Arc,
};

use common::{
    glam::{UVec2, Vec2, Vec3, Vec4},
    hecs::{self, Entity},
    log,
};
mod beacon;
mod combat_state;
mod job;
mod transform;
mod viking;
pub use beacon::Beacon;
pub use combat_state::CombatState;
pub use job::{Job, JobState};
use serde::{Deserialize, Serialize};
pub use transform::Transform;
pub use viking::{BrainwashState, Viking};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, PartialOrd, Copy)]
pub struct GameTime(f64);

impl PartialOrd<f32> for GameTime {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.as_secs_f32().partial_cmp(other)
    }
}

impl PartialEq<f32> for GameTime {
    fn eq(&self, other: &f32) -> bool {
        self.as_secs_f32() == *other
    }
}

impl Display for GameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} seconds", self.0)
    }
}

impl Sub for GameTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl GameTime {
    pub fn as_secs_f32(&self) -> f32 {
        self.0 as _
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.0 as _
    }

    pub fn add(&mut self, actual_delta: f64) {
        self.0 += actual_delta
    }
}

impl AddAssign<GameTime> for f32 {
    fn add_assign(&mut self, rhs: GameTime) {
        *self += rhs.as_secs_f32()
    }
}

impl From<f64> for GameTime {
    fn from(value: f64) -> Self {
        GameTime(value)
    }
}

impl From<f32> for GameTime {
    fn from(value: f32) -> Self {
        GameTime(value as _)
    }
}

impl From<GameTime> for f32 {
    fn from(value: GameTime) -> Self {
        value.0 as _
    }
}

impl From<GameTime> for f64 {
    fn from(value: GameTime) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLTFAsset {
    pub name: String,
}

impl GLTFAsset {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Targeted;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetIndicator(pub hecs::Entity);

/// tag component to indicate that we'd like a collider based on our geometry, please
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collider {
    pub y_offset: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parent {
    pub entity: Entity,
    pub offset: Transform,
}

impl Parent {
    pub fn new(entity: Entity, offset: Transform) -> Self {
        Self { entity, offset }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dave {
    pub energy: usize,
    pub last_brainwash_time: GameTime,
    pub last_energy_drain_time: GameTime,
}

impl Dave {
    pub fn new(energy: usize) -> Self {
        Self {
            energy,
            ..Default::default()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Resource {
    RawIron,
    Iron,
    Paperclip,
    Food,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
}

impl Info {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selected;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Task {
    Gather(Resource),
    Smelt,
    MakePaperclips,
    Construction,
}

impl Task {
    pub const fn resource_produced(&self) -> Option<Resource> {
        match self {
            Task::Gather(resource) => Some(*resource),
            Task::Smelt => Some(Resource::Iron),
            Task::MakePaperclips => Some(Resource::Paperclip),
            Task::Construction => None,
        }
    }

    pub const fn resource_consumed(&self) -> Option<Resource> {
        match self {
            Task::Gather(resource) => Some(*resource),
            Task::Smelt => Some(Resource::RawIron),
            Task::MakePaperclips => Some(Resource::Iron),
            _ => None,
        }
    }

    pub const fn work_duration(&self) -> f32 {
        match self {
            Task::Gather(_) => 4.,
            Task::Smelt => 4.,
            Task::MakePaperclips => 5.,
            Task::Construction => 10.,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Building {
    House,
    PlaceOfWork(WorkplaceType),
}

impl Building {
    pub fn place_of_work(&self) -> Option<WorkplaceType> {
        match self {
            Building::PlaceOfWork(workplace) => Some(*workplace),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorkplaceType {
    Mine,
    Forge,
    Factory,
    ConstructionSite,
    Farm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOfWork {
    pub place_type: WorkplaceType,
    pub task: Task,
    pub worker_capacity: usize,
    pub workers: VecDeque<hecs::Entity>,
}

impl PlaceOfWork {
    pub fn mine() -> PlaceOfWork {
        PlaceOfWork {
            place_type: WorkplaceType::Mine,
            task: Task::Gather(Resource::RawIron),
            worker_capacity: 5,
            workers: Default::default(),
        }
    }

    pub fn forge() -> PlaceOfWork {
        PlaceOfWork {
            place_type: WorkplaceType::Forge,
            task: Task::Smelt,
            worker_capacity: 2,
            workers: Default::default(),
        }
    }

    pub fn factory() -> PlaceOfWork {
        PlaceOfWork {
            place_type: WorkplaceType::Factory,
            task: Task::MakePaperclips,
            worker_capacity: 1,
            workers: Default::default(),
        }
    }

    pub fn construction_site() -> PlaceOfWork {
        PlaceOfWork {
            place_type: WorkplaceType::ConstructionSite,
            task: Task::Construction,
            worker_capacity: 5,
            workers: Default::default(),
        }
    }

    pub fn farm() -> PlaceOfWork {
        PlaceOfWork {
            place_type: WorkplaceType::Farm,
            task: Task::Gather(Resource::Food),
            worker_capacity: 4,
            workers: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingGhost {
    pub target_building: Building,
}

impl BuildingGhost {
    pub fn new(target_building: Building) -> Self {
        Self { target_building }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionSite {
    pub target_building: Building,
    pub construction_progress: f32,
}

impl ConstructionSite {
    pub fn new(target_building: Building) -> Self {
        Self {
            target_building,
            construction_progress: 0.,
        }
    }

    pub fn resources_required(&self) -> (usize, Resource) {
        match self.target_building {
            Building::PlaceOfWork(WorkplaceType::Forge) => (1, Resource::RawIron),
            Building::PlaceOfWork(WorkplaceType::Factory) => (1, Resource::Iron),
            Building::House => (2, Resource::Iron),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Storage;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    inner: HashMap<Resource, usize>,
}

impl Inventory {
    pub fn new<H: Into<HashMap<Resource, usize>>>(inner: H) -> Self {
        Self {
            inner: inner.into(),
        }
    }

    pub fn take(&mut self, amount: usize, resource: Resource) -> Option<usize> {
        log::trace!("Attempting to take {amount} {resource:?} from {self:?}..");
        if let Some(remaining) = self.inner.get_mut(&resource) {
            if *remaining == 0 {
                log::trace!("None left!");
                return None;
            }
            // TODO do this properly
            *remaining = remaining.checked_sub(amount).unwrap_or_default();
            return Some(amount);
        }
        log::trace!("No {resource:?} found!");

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialOverrides {
    pub base_colour_factor: Vec4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub value: usize,
    pub last_taken_time: GameTime,
    pub last_regen_time: GameTime,
}

impl Health {
    pub fn new(value: usize) -> Self {
        Self {
            value,
            last_taken_time: GameTime::default(),
            last_regen_time: GameTime::default(),
        }
    }

    pub fn take(&mut self, amount: usize, now: GameTime) -> usize {
        self.value = self.value.saturating_sub(amount);
        self.last_taken_time = now;
        self.value
    }

    pub fn add(&mut self, amount: usize, now: GameTime) -> usize {
        self.value = (self.value + amount).min(100);
        self.last_regen_time = now;
        self.value
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct House {
    pub occupants: Vec<Entity>,
    pub capacity: usize,
}

impl House {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            ..Default::default()
        }
    }

    pub fn has_capacity(&self) -> bool {
        self.occupants.len() < self.capacity
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
/// Various needs of humans. You want these to be zero.
pub struct HumanNeeds {
    pub hunger: usize,
    pub sleep: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum RestState {
    #[default]
    Idle,
    GettingFood(Entity),
    Eating(f32),
    GoingHome(Entity),
    Sleeping(f32),
    NoFoodAvailable,
    NoHomeAvailable,
}
