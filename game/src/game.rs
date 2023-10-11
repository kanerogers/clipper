use crate::{
    clock::Clock, ecs::SerialisationContext, systems::PhysicsContext, time::Time, HumanNeedsState,
    Input,
};
use common::{
    anyhow::{self, anyhow},
    glam::Vec3,
    hecs::{self, RefMut},
    rapier3d::prelude::Ray,
    serde::{ser::SerializeMap, Serialize},
    serde_json, winit, Camera, Line,
};
use components::{Dave, GameTime, Storage, Transform};

pub struct Game {
    pub world: hecs::World,
    pub time: Time,
    pub dave: hecs::Entity,
    pub input: Input,
    pub camera: Camera,
    pub physics_context: PhysicsContext,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub debug_lines: Vec<Line>,
    pub last_ray: Option<Ray>,
    pub game_over: bool,
    pub clock: Clock,
    pub human_needs_state: HumanNeedsState,
    pub total_deaths: usize,
    pub serialisation_context: SerialisationContext,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            world: Default::default(),
            time: Default::default(),
            dave: hecs::Entity::DANGLING,
            input: Default::default(),
            camera: Default::default(),
            physics_context: Default::default(),
            window_size: Default::default(),
            debug_lines: Default::default(),
            last_ray: None,
            game_over: false,
            clock: Default::default(),
            human_needs_state: HumanNeedsState::default(),
            total_deaths: 0,
            serialisation_context: Default::default(),
        }
    }
}

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: common::serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(8))?;
        // TODO: handle error nicer here
        let serialised_world = self
            .serialisation_context
            .serialise_world(&self.world)
            .unwrap();
        map.serialize_entry("world", &serialised_world)?;
        map.serialize_entry("time", &self.time)?;
        map.serialize_entry("dave", &self.dave)?;
        map.serialize_entry("camera", &self.camera)?;
        map.serialize_entry("game_over", &self.game_over)?;
        map.serialize_entry("clock", &self.clock)?;
        map.serialize_entry("human_needs_state", &self.human_needs_state)?;
        map.serialize_entry("total_deaths", &self.total_deaths)?;

        map.end()
    }
}

// generic deserialisation is hard, can't be bothered
// see: `[Game::from_json]` for lazy version
// impl Deserialize for Game {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: common::serde::Deserializer<'de> {
//         todo!()
//     }
// }

impl Game {
    pub fn resized(&mut self, window_size: winit::dpi::PhysicalSize<u32>) {
        self.window_size = window_size;
        self.camera.resized(window_size);
    }

    pub fn dave_position(&self) -> Vec3 {
        self.position_of(self.dave)
    }

    /// **panics**
    ///
    /// This method will panic if the entity does not exist.
    pub fn position_of(&self, entity: hecs::Entity) -> Vec3 {
        let world = &self.world;
        world.get::<&Transform>(entity).unwrap().position
    }

    pub fn dave(&self) -> RefMut<Dave> {
        self.world.get::<&mut Dave>(self.dave).unwrap()
    }

    pub fn command_buffer(&self) -> hecs::CommandBuffer {
        hecs::CommandBuffer::new()
    }

    pub fn run_command_buffer(&mut self, mut command_buffer: hecs::CommandBuffer) {
        command_buffer.run_on(&mut self.world);
    }

    pub fn get<'a, C: hecs::Component>(&'a self, entity: hecs::Entity) -> RefMut<'_, C> {
        self.world.get::<&'a mut C>(entity).unwrap()
    }

    pub fn storage(&self) -> hecs::Entity {
        self.world
            .query::<()>()
            .with::<&Storage>()
            .iter()
            .next()
            .unwrap()
            .0
    }

    pub fn now(&self) -> GameTime {
        self.time.now()
    }

    pub fn from_json(value: &serde_json::Value) -> Result<Game, anyhow::Error> {
        let serialisation_context = SerialisationContext::default();
        let values = GameValues::from_value(value)
            .ok_or_else(|| anyhow!("Invalid JSON: {}", value.to_string()))?;
        let world = serialisation_context.deserialise_world(&values.world)?;

        let time = serde_json::from_value(values.time)?;
        let dave = serde_json::from_value(values.dave)?;
        let camera = serde_json::from_value(values.camera)?;
        let game_over = serde_json::from_value(values.game_over)?;
        let clock = serde_json::from_value(values.clock)?;
        let human_needs_state = serde_json::from_value(values.human_needs_state)?;
        let total_deaths = serde_json::from_value(values.total_deaths)?;

        let game = Game {
            world,
            time,
            dave,
            camera,
            game_over,
            clock,
            human_needs_state,
            total_deaths,
            serialisation_context,
            ..Default::default()
        };
        Ok(game)
    }
}

pub struct GameValues {
    world: serde_json::Value,
    time: serde_json::Value,
    camera: serde_json::Value,
    game_over: serde_json::Value,
    clock: serde_json::Value,
    dave: serde_json::Value,
    human_needs_state: serde_json::Value,
    total_deaths: serde_json::Value,
}
impl GameValues {
    fn from_value(value: &serde_json::Value) -> Option<Self> {
        let map = value.as_object()?;
        Some(Self {
            world: map.get("world")?.clone(),
            time: map.get("time")?.clone(),
            camera: map.get("camera")?.clone(),
            game_over: map.get("game_over")?.clone(),
            dave: map.get("dave")?.clone(),
            clock: map.get("clock")?.clone(),
            human_needs_state: map.get("human_needs_state")?.clone(),
            total_deaths: map.get("total_deaths")?.clone(),
        })
    }
}
