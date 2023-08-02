mod input;
mod systems;
pub mod time;

use common::{
    bitflags::bitflags,
    glam::{Quat, Vec2, Vec3},
    hecs, rand,
    rapier3d::prelude::Ray,
    winit::{self},
    Camera, GUIState, HumanInfo, Line, PlaceOfWorkInfo, SelectedItemInfo,
};

use components::{
    Beacon, Collider, Dave, GLTFAsset, Human, HumanState, Info, Inventory, PlaceOfWork, Resource,
    Selected, Storage, Transform, Velocity,
};
use std::collections::VecDeque;
use systems::{
    beacons, click_system, dave_controller, from_na, physics, update_human_colour,
    update_human_position, update_human_state, PhysicsContext,
};
use time::Time;

pub const PLAYER_SPEED: f32 = 7.;
pub const CAMERA_ZOOM_SPEED: f32 = 10.;
pub const CAMERA_ROTATE_SPEED: f32 = 3.;
const RENDER_DEBUG_LINES: bool = false;

// required due to reasons
#[no_mangle]
pub fn init() -> Game {
    Game::new()
}

#[no_mangle]
pub fn tick(game: &mut Game, gui_state: &mut GUIState) {
    while game.time.start_update() {
        game.debug_lines.clear();
        process_gui_command_queue(game, &mut gui_state.command_queue);
        update_camera(game);
        click_system(game);
        dave_controller(game);
        update_human_state(game);
        update_human_position(game);
        update_human_colour(game);
        physics(game);
        beacons(game);
        update_gui_state(game, gui_state);
    }

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
}

fn process_gui_command_queue(game: &mut Game, command_queue: &mut VecDeque<common::GUICommand>) {
    let world = &mut game.world;
    for command in command_queue.drain(..) {
        println!("Processing command: {command:?}");

        match command {
            common::GUICommand::SetWorkerCount(place_of_work_entity, desired_worker_count) => {
                let mut place_of_work =
                    world.get::<&mut PlaceOfWork>(place_of_work_entity).unwrap();
                let current_workers = place_of_work.workers.len();
                if desired_worker_count > current_workers {
                    if let Some(worker_entity) = find_available_worker(world) {
                        place_of_work.workers.push_front(worker_entity);
                        let mut worker = world.get::<&mut Human>(worker_entity).unwrap();
                        worker.assign_place_of_work(place_of_work_entity);
                    }
                } else {
                    if let Some(worker_entity) = place_of_work.workers.pop_back() {
                        let mut worker = world.get::<&mut Human>(worker_entity).unwrap();
                        worker.unassign_work();
                    }
                }
            }
            common::GUICommand::Liquify(entity) => world.despawn(entity).unwrap(),
        }
    }
}

fn find_available_worker(world: &hecs::World) -> Option<hecs::Entity> {
    let mut query = world.query::<&Human>();
    for (entity, human) in query.iter() {
        if human.state == HumanState::AwaitingAssignment {
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
        }
    }
}

impl Game {
    pub fn new() -> Self {
        let mut camera = Camera::default();
        let mut world = hecs::World::new();

        // dave
        let dave = world.spawn((
            GLTFAsset::new("droid.glb"),
            Dave::default(),
            Transform::from_position([0., 2., 0.].into()),
            Velocity::default(),
            Info::new("DAVE"),
        ));

        // terrain
        world.spawn((
            Transform::default(),
            Info::new("Ground"),
            GLTFAsset::new("environment.glb"),
        ));

        const STARTING_HUMANS: usize = 10;
        for i in 0..STARTING_HUMANS {
            let x = (rand::random::<f32>() * 50.) - 25.;
            let z = (rand::random::<f32>() * 50.) - 25.;
            world.spawn((
                Collider::default(),
                GLTFAsset::new("viking_1.glb"),
                Human::default(),
                Transform::from_position([x, 1., z].into()),
                Velocity::default(),
                Info::new(format!("Human {i}")),
            ));
        }

        // beacon
        world.spawn((
            Collider::default(),
            GLTFAsset::new("ship.glb"),
            Beacon::default(),
            Transform::default(),
            Info::new("Ship"),
            Inventory::new([]),
            Storage,
        ));

        // mine
        world.spawn((
            Collider::default(),
            GLTFAsset::new("mine.glb"),
            Transform::from_position([30.0, 0.0, 0.0].into()),
            Velocity::default(),
            PlaceOfWork::mine(),
            Inventory::new([(Resource::RawIron, 5000)]),
            Info::new("Mine"),
        ));

        // forge
        world.spawn((
            Collider::default(),
            GLTFAsset::new("forge.glb"),
            Transform::from_position([-30., 0.0, 0.0].into()),
            Velocity::default(),
            PlaceOfWork::forge(),
            Inventory::new([]),
            Info::new("Forge"),
        ));

        // factory
        world.spawn((
            Collider::default(),
            GLTFAsset::new("factory.glb"),
            Transform::from_position([20., 0.0, 30.0].into()),
            Velocity::default(),
            PlaceOfWork::factory(),
            Inventory::new([]),
            Info::new("Factory"),
        ));

        camera.position.y = 3.;
        camera.position.z = 12.;
        camera.distance = 50.;
        camera.desired_distance = camera.distance;
        camera.start_distance = camera.distance;
        Self {
            camera,
            dave,
            world,
            ..Default::default()
        }
    }

    pub fn resized(&mut self, window_size: winit::dpi::PhysicalSize<u32>) {
        self.window_size = window_size;
        self.camera.resized(window_size);
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
    gui_state.idle_workers = game
        .world
        .query_mut::<&Human>()
        .into_iter()
        .filter(|(_, h)| h.state == HumanState::AwaitingAssignment)
        .count();

    gui_state.paperclips = game
        .world
        .query::<&Inventory>()
        .with::<&Storage>()
        .iter()
        .map(|(_, i)| i.amount_of(Resource::Paperclip))
        .sum();

    if let Some((entity, human)) = game
        .world
        .query::<&Human>()
        .with::<&Selected>()
        .iter()
        .next()
    {
        let place_of_work = human
            .place_of_work
            .map(|p| game.world.get::<&PlaceOfWork>(p).unwrap().place_type);
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::Human(HumanInfo {
                inventory: format!("{:?}", human.inventory),
                name: "Boris".into(),
                state: format!("{:?}", human.state),
                place_of_work: format!("{place_of_work:?}"),
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
