use crate::{clock::Clock, systems::PhysicsContext, time::Time, HumanNeedsState, Input};
use common::{
    glam::Vec3,
    hecs::{self, RefMut},
    rapier3d::prelude::Ray,
    winit, Camera, Line,
};
use components::{Dave, Storage, Transform};

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
        }
    }
}

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
}
