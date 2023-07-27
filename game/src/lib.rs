pub mod beacon;
pub mod components;
mod input;
mod systems;
pub mod time;

pub use common::Mesh;
use common::{
    bitflags::bitflags,
    glam::{Quat, Vec2, Vec3},
    hecs, rand,
    winit::{self},
    Camera, GUIState, Geometry, Material,
};
use components::{Dave, Human, HumanState, Transform, Velocity};
use std::f32::consts::TAU;
use systems::{beacons, click_system, dave_controller, humans, physics, PhysicsContext};
use time::Time;

use crate::beacon::Beacon;

pub const PLAYER_SPEED: f32 = 7.;
pub const CAMERA_ZOOM_SPEED: f32 = 10.;
pub const CAMERA_ROTATE_SPEED: f32 = 3.;

// required due to reasons
#[no_mangle]
pub fn init() -> Game {
    Game::new()
}

#[no_mangle]
pub fn tick(game: &mut Game, gui_state: &mut GUIState) -> Vec<Mesh> {
    while game.time.start_update() {
        update_camera(game);
        click_system(game);
        dave_controller(game);
        humans(game);
        physics(game);
        beacons(game);
        update_gui_state(game, gui_state);
    }

    game.meshes()
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
        }
    }
}

impl Game {
    pub fn new() -> Self {
        let mut camera = Camera::default();
        let mut world = hecs::World::new();

        // dave
        let dave = world.spawn((
            Dave::default(),
            Geometry::Sphere,
            Material::from_colour([0., 0., 1.].into()),
            Transform::from_translation([0., 2., 0.].into()),
            Velocity::default(),
        ));

        // terrain
        world.spawn(get_grid());

        const STARTING_HUMANS: usize = 10;
        for _ in 0..STARTING_HUMANS {
            let x = (rand::random::<f32>() * 50.) - 25.;
            let z = (rand::random::<f32>() * 50.) - 25.;
            world.spawn((
                Human::default(),
                Geometry::Cube,
                Material::from_colour([0., 1., 0.].into()),
                Transform::from_translation([x, 1., z].into()),
                Velocity::default(),
            ));
        }

        // beacon
        world.spawn((
            Beacon::default(),
            Geometry::Cube,
            Material::from_colour([0., 0., 0.].into()),
            Transform::new(
                [0.0, 0.0, 0.0].into(),
                Default::default(),
                [2.0, 20., 2.0].into(),
            ),
            Velocity::default(),
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

    pub fn meshes(&self) -> Vec<Mesh> {
        self.world
            .query::<(&Geometry, &Transform, &Material)>()
            .into_iter()
            .map(|(_, (geometry, transform, material))| Mesh {
                geometry: *geometry,
                texture_id: material.texture_id,
                transform: transform.into(),
                colour: Some(material.colour),
            })
            .collect()
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

#[derive(Clone, Debug)]
pub struct Input {
    pub keyboard_state: Keys,
    pub camera_zoom: f32,
    pub click_position: Option<Vec2>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keyboard_state: Default::default(),
            camera_zoom: 0.,
            click_position: None,
        }
    }
}

impl Input {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

fn get_grid() -> (Transform, Material, Geometry) {
    let plane_rotation = Quat::from_rotation_x(TAU / 4.0); // 90 degrees
    let grid_size = 255;

    (
        Transform {
            scale: [grid_size as f32, grid_size as f32, 1. as f32].into(),
            rotation: plane_rotation,
            ..Default::default()
        },
        Material::from_colour(rgb_to_vec(11, 102, 35)),
        Geometry::Plane,
    )
}

fn rgb_to_vec(r: usize, g: usize, b: usize) -> Vec3 {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255.].into()
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
    gui_state.workers = game
        .world
        .query_mut::<&Human>()
        .into_iter()
        .filter(|(_, h)| h.state == HumanState::Working)
        .count();
}
