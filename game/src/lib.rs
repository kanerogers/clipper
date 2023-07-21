pub mod beacon;
pub mod dave;
pub mod human;
pub mod time;

use beacon::Beacon;
pub use common::Mesh;
use common::{
    bitflags::bitflags,
    glam::{Affine3A, Quat, Vec3},
    Camera, GUIState, Geometry,
};
use dave::Dave;
use human::{humans, Human};
use std::f32::consts::TAU;
use time::Time;

pub const PLAYER_SPEED: f32 = 7.;
pub const CAMERA_ZOOM_SPEED: f32 = 10.;
pub const CAMERA_ROTATE_SPEED: f32 = 3.;

#[derive(Clone, Debug, Default)]
pub struct Game {
    pub time: Time,
    pub dave: Dave,
    pub input: Input,
    pub camera: Camera,
    pub terrain: Vec<Mesh>,
    pub humans: Vec<Human>,
    pub beacons: Vec<Beacon>,
    pub gui_state: GUIState,
}

impl Game {
    pub fn new() -> Self {
        let mut camera = Camera::default();
        let dave = Dave::default();
        camera.position.y = 3.;
        camera.position.z = 12.;
        camera.focus_point = dave.position;
        camera.distance = 10.;
        camera.desired_distance = camera.distance;
        camera.start_distance = camera.distance;
        Self {
            camera,
            dave,
            terrain: get_grid(),
            humans: vec![
                Human::new([0., 1., 4.].into()),
                Human::new([-3., 1., 0.].into()),
                Human::new([0., 1., 2.].into()),
            ],
            beacons: vec![Beacon::new([15., 4., 0.].into())],
            ..Default::default()
        }
    }

    pub fn meshes(&self) -> Vec<Mesh> {
        let mut meshes = self.terrain.clone();
        meshes.push(self.dave.mesh);
        meshes.extend(self.humans.iter().map(|h| h.mesh));
        meshes.extend(self.beacons.iter().map(|h| h.mesh));
        meshes
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
}

impl Default for Input {
    fn default() -> Self {
        Self {
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

fn get_grid() -> Vec<Mesh> {
    let plane_rotation = Quat::from_rotation_x(TAU / 4.0); // 90 degrees
    let mut meshes = Vec::new();

    let grid_size = 255;

    meshes.push(Mesh {
        geometry: Geometry::Plane,
        transform: Affine3A::from_scale_rotation_translation(
            [grid_size as f32, grid_size as f32, 1. as f32].into(),
            plane_rotation,
            Default::default(),
        ),
        colour: Some(rgb_to_vec(11, 102, 35)),
        ..Default::default()
    });

    meshes
}

fn rgb_to_vec(r: usize, g: usize, b: usize) -> Vec3 {
    [r as f32 / 255., g as f32 / 255., b as f32 / 255.].into()
}

pub fn dave(game: &mut Game) {
    let dt = game.time.delta();
    game.dave.update(dt, &game.input, &game.camera.transform());
}
pub fn update_camera(game: &mut Game) {
    let camera = &mut game.camera;
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

#[no_mangle]
pub fn tick(game: &mut Game) -> Vec<Mesh> {
    while game.time.start_update() {
        game.camera.target = game.dave.position;
        update_camera(game);
        dave(game);
        humans(game);
    }

    game.meshes()
}

// required due to reasons
#[no_mangle]
pub fn init() -> Game {
    Game::new()
}
