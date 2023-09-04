pub mod clock;
mod config;
mod init;
mod input;
mod systems;
pub mod time;
use clock::Clock;
use common::{
    bitflags::bitflags,
    glam::{Quat, Vec2, Vec3},
    hecs::{self, RefMut},
    log,
    rapier3d::prelude::Ray,
    winit::{self},
    Camera, GUICommand, GUIState, Line, PlaceOfWorkInfo, SelectedItemInfo, VikingInfo,
    BUILDING_TYPE_FACTORY, BUILDING_TYPE_FORGE,
};
use components::{
    BrainwashState, BuildingGhost, Collider, ConstructionSite, Dave, GLTFAsset, Health, Inventory,
    Job, JobState, MaterialOverrides, PlaceOfWork, Resource, Selected, Storage, Task, Transform,
    Viking, WorkplaceType,
};
use config::{BUILDING_TRANSPARENCY, MAX_ENERGY, MAX_HEALTH};
use init::init_game;
use std::collections::VecDeque;
use systems::{
    beacons, brainwash::brainwash_system, click_system, combat::combat_system,
    construction::construction_system, dave_controller,
    find_brainwash_target::update_brainwash_target, from_na, game_over::game_over_system, physics,
    regen::regen_system, target_indicator::target_indicator_system,
    transform_hierarchy::transform_hierarchy_system, update_position::update_position_system,
    viking_velocity::update_viking_velocity, viking_work::viking_work_system, PhysicsContext,
};
use time::Time;

pub const PLAYER_SPEED: f32 = 7.;
pub const CAMERA_ZOOM_SPEED: f32 = 10.;
pub const CAMERA_ROTATE_SPEED: f32 = 3.;
const RENDER_DEBUG_LINES: bool = false;

// required due to reasons
#[no_mangle]
pub fn init() -> Game {
    init::init_game()
}

#[no_mangle]
pub fn tick(game: &mut Game, gui_state: &mut GUIState) -> bool {
    let mut need_restart = false;
    while game.time.start_update() {
        game.clock.advance(game.time.delta());
        game.debug_lines.clear();
        need_restart = process_gui_command_queue(game, &mut gui_state.command_queue);
        update_camera(game);
        game_over_system(game);

        if !game.game_over {
            click_system(game);
            dave_controller(game);
            update_brainwash_target(game);
            brainwash_system(game);
            combat_system(game);
            regen_system(game);
            construction_system(game);
        }

        target_indicator_system(game);
        viking_work_system(game);
        update_viking_velocity(game);
        physics(game);
        beacons(game);
        update_position_system(game);
        transform_hierarchy_system(game);
        reset_mouse_clicks(&mut game.input.mouse_state);
    }

    update_gui_state(game, gui_state);

    if let Some(last_ray) = game.last_ray {
        let origin = from_na(last_ray.origin);
        let direction: Vec3 = from_na(last_ray.dir);
        let end = origin + direction * 100.;

        game.debug_lines.push(Line {
            start: origin,
            end,
            colour: [1., 0., 1.].into(),
        });
    }

    if !RENDER_DEBUG_LINES {
        game.debug_lines.clear();
    }

    need_restart
}

fn process_gui_command_queue(game: &mut Game, command_queue: &mut VecDeque<GUICommand>) -> bool {
    let world = &game.world;
    let mut command_buffer = hecs::CommandBuffer::new();
    let mut need_restart = false;

    for command in command_queue.drain(..) {
        match command {
            GUICommand::SetWorkerCount(place_of_work_entity, desired_worker_count) => {
                set_worker_count(
                    world,
                    place_of_work_entity,
                    desired_worker_count,
                    &mut command_buffer,
                );
            }
            GUICommand::Liquify(entity) => command_buffer.despawn(entity),
            GUICommand::Restart => {
                need_restart = true;
            }
            GUICommand::ConstructBuilding(building_name) => {
                show_ghost_building(building_name, world, &mut command_buffer);
            }
        }
    }

    game.run_command_buffer(command_buffer);

    if need_restart {
        *game = init_game();
    }

    need_restart
}

fn set_worker_count(
    world: &hecs::World,
    place_of_work_entity: hecs::Entity,
    desired_worker_count: usize,
    command_buffer: &mut hecs::CommandBuffer,
) {
    let mut place_of_work = world.get::<&mut PlaceOfWork>(place_of_work_entity).unwrap();
    let current_workers = place_of_work.workers.len();
    if desired_worker_count > current_workers {
        if let Some(worker_entity) = find_available_worker(world) {
            place_of_work.workers.push_front(worker_entity);
            let mut job = Job::new(place_of_work_entity);

            // TODO: this is inelegant
            if place_of_work.task == Task::Construction {
                let storage = world.query::<&Storage>().iter().next().unwrap().0;
                let construction_site = world
                    .get::<&ConstructionSite>(place_of_work_entity)
                    .unwrap();
                job.state =
                    JobState::FetchingResource(construction_site.resources_required().1, storage);
            }
            command_buffer.insert_one(worker_entity, job);
        }
    } else {
        if let Some(worker_entity) = place_of_work.workers.pop_back() {
            command_buffer.remove_one::<Job>(worker_entity);
        }
    }
}

fn show_ghost_building(
    building_name: &str,
    world: &hecs::World,
    command_buffer: &mut hecs::CommandBuffer,
) {
    let (building_type, asset_name) = match building_name {
        BUILDING_TYPE_FACTORY => (WorkplaceType::Factory, "factory.glb"),
        BUILDING_TYPE_FORGE => (WorkplaceType::Forge, "forge.glb"),
        _ => {
            log::error!("Attempted to build unknown building type {building_name}");
            return;
        }
    };

    // Remove any existing ghosts
    for (entity, _) in world.query::<()>().with::<&BuildingGhost>().iter() {
        command_buffer.despawn(entity);
    }

    command_buffer.spawn((
        BuildingGhost::new(building_type),
        GLTFAsset::new(asset_name),
        Transform::default(),
        Collider::default(),
        MaterialOverrides {
            base_colour_factor: [1., 1., 1., BUILDING_TRANSPARENCY].into(),
        },
    ));
}

fn find_available_worker(world: &hecs::World) -> Option<hecs::Entity> {
    let mut query = world.query::<&Viking>();
    for (entity, viking) in query.iter() {
        if viking.brainwash_state == BrainwashState::Brainwashed {
            return Some(entity);
        }
    }

    None
}

#[no_mangle]
pub fn handle_winit_event(game: &mut Game, event: winit::event::WindowEvent) {
    input::handle_winit_event(game, event);
}

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

pub struct ECS<'a> {
    pub world: &'a hecs::World,
}

impl ECS<'_> {
    pub fn position_of(&self, entity: hecs::Entity) -> Vec3 {
        let world = &self.world;
        world.get::<&Transform>(entity).unwrap().position
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Keys: u8 {
        const W = 0b00000001;
        const A = 0b00000010;
        const S = 0b00000100;
        const D = 0b00001000;
        const Q = 0b00010000;
        const E = 0b00100000;
        const C = 0b01000000;
        const Space = 0b10000000;
    }
}

impl Keys {
    pub fn as_axis(&self, negative: Keys, positive: Keys) -> f32 {
        let negative = self.contains(negative) as i8 as f32;
        let positive = self.contains(positive) as i8 as f32;
        positive - negative
    }
}

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub position: Option<Vec2>,
    pub left_click_state: ClickState,
    pub right_click_state: ClickState,
    pub middle_click_state: ClickState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ClickState {
    #[default]
    Released,
    Down,
    JustReleased,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub keyboard_state: Keys,
    pub mouse_state: MouseState,
    pub camera_zoom: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            mouse_state: Default::default(),
            keyboard_state: Default::default(),
            camera_zoom: 0.,
        }
    }
}

impl Input {
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    pub fn is_pressed(&self, key: Keys) -> bool {
        self.keyboard_state.contains(key)
    }
}

pub fn update_camera(game: &mut Game) {
    let camera = &mut game.camera;
    camera.target = game.world.get::<&Transform>(game.dave).unwrap().position;
    let input = &game.input;
    let dt = game.time.delta();

    let focus_radius = 1.0;
    let focus_centering = 0.5;
    let distance_to_target = camera.target.distance(camera.focus_point);

    let mut t = 1.0;
    if distance_to_target > 0.01 {
        t = ((1. - focus_centering) as f32).powf(dt);
    }
    if distance_to_target > focus_radius {
        t = t.min(focus_radius / distance_to_target);
    }
    camera.focus_point = camera.target.lerp(camera.focus_point, t);

    let camera_rotate = input.keyboard_state.as_axis(Keys::E, Keys::Q);
    camera.yaw += camera_rotate * CAMERA_ROTATE_SPEED * dt;

    set_camera_distance(input, camera, dt);

    camera.pitch = -45_f32.to_radians();
    let look_rotation = Quat::from_euler(common::glam::EulerRot::YXZ, camera.yaw, camera.pitch, 0.);
    let look_direction = look_rotation * Vec3::NEG_Z;
    let look_position = camera.focus_point - look_direction * camera.distance;

    camera.position = look_position;
}

fn set_camera_distance(input: &Input, camera: &mut Camera, dt: f32) {
    if input.camera_zoom.abs() > 0. {
        camera.start_distance = camera.distance;
        camera.desired_distance += input.camera_zoom;
        camera.desired_distance = camera.desired_distance.clamp(5., 50.);
    }

    let current_delta = camera.desired_distance - camera.distance;

    let epsilon = 0.01;
    if current_delta.abs() > epsilon {
        camera.distance += current_delta * CAMERA_ZOOM_SPEED * dt;
    } else {
        camera.distance = camera.desired_distance;
    }
}

fn update_gui_state(game: &mut Game, gui_state: &mut GUIState) {
    gui_state.game_over = game.game_over;
    gui_state.idle_workers = game
        .world
        .query_mut::<&Viking>()
        .without::<&Job>()
        .into_iter()
        .filter(|(_, h)| h.brainwash_state == BrainwashState::Brainwashed)
        .count();

    gui_state.paperclips = game
        .world
        .query::<&Inventory>()
        .with::<&Storage>()
        .iter()
        .map(|(_, i)| i.amount_of(Resource::Paperclip))
        .sum();

    {
        let dave = game.world.get::<&Dave>(game.dave).unwrap();
        let health = game.world.get::<&Health>(game.dave).unwrap();
        gui_state.bars.health_percentage = health.value as f32 / MAX_HEALTH as f32;
        gui_state.bars.energy_percentage = dave.energy as f32 / MAX_ENERGY as f32;
    }

    gui_state.clock = format!("{}", game.clock);
    gui_state.clock_description = if game.clock.is_work_time() {
        "Work Time".to_string()
    } else {
        "Rest Time".to_string()
    };

    if let Some((entity, (viking, job))) = game
        .world
        .query::<(&Viking, Option<&Job>)>()
        .with::<&Selected>()
        .iter()
        .next()
    {
        let place_of_work = job.map(|j| {
            game.world
                .get::<&PlaceOfWork>(j.place_of_work)
                .unwrap()
                .place_type
        });
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::Viking(VikingInfo {
                inventory: format!("{:?}", viking.inventory),
                name: "Boris".into(),
                state: format!("{}", viking.brainwash_state),
                place_of_work: format!("{place_of_work:?}"),
                intelligence: viking.intelligence,
                strength: viking.strength,
                stamina: viking.stamina,
            }),
        ));
        return;
    }

    if let Some((entity, (place_of_work, inventory))) = game
        .world
        .query_mut::<(&PlaceOfWork, &Inventory)>()
        .with::<&Selected>()
        .into_iter()
        .next()
    {
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::PlaceOfWork(PlaceOfWorkInfo {
                name: format!("{:?}", place_of_work.place_type),
                task: format!("{:?}", place_of_work.task),
                workers: place_of_work.workers.len(),
                max_workers: place_of_work.worker_capacity,
                stock: format!("{inventory:?}"),
            }),
        ));
        return;
    }

    if let Some((entity, (_, inventory))) = game
        .world
        .query_mut::<(&Storage, &Inventory)>()
        .with::<&Selected>()
        .into_iter()
        .next()
    {
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::PlaceOfWork(PlaceOfWorkInfo {
                name: "Storage".into(),
                task: "Storing things".into(),
                workers: 0,
                max_workers: 0,
                stock: format!("{inventory:?}"),
            }),
        ));
        return;
    }

    // nothing was selected!
    gui_state.selected_item = None;
}

fn reset_mouse_clicks(mouse_state: &mut crate::MouseState) {
    match mouse_state.left_click_state {
        ClickState::JustReleased => mouse_state.left_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.right_click_state {
        ClickState::JustReleased => mouse_state.right_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.middle_click_state {
        ClickState::JustReleased => mouse_state.middle_click_state = ClickState::Released,
        _ => {}
    };
}
