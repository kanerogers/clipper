pub mod time;

use common::{
    glam::{Affine3A, Quat, Vec3},
    Geometry, Mesh,
};
use std::f32::consts::TAU;
use time::Time;

static PLAYER_SPEED: f32 = 5.;

#[derive(Clone, Debug, Default)]
pub struct Game {
    pub meshes: Vec<Mesh>,
    pub time: Time,
    pub dave: Dave,
    pub input: Input,
}

// objectively terrible input handling
#[derive(Clone, Debug)]
pub struct Input {
    pub movement: Vec3,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            movement: Vec3::ZERO,
        }
    }
}

impl Input {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug)]
pub struct Dave {
    pub position: Vec3,
    pub velocity: Vec3,
}

impl Dave {
    pub fn update(&mut self, dt: f32, input: &Input) {
        let movement = input.movement.normalize();
        self.velocity = if !movement.is_nan() {
            movement
        } else {
            Vec3::ZERO
        };

        let displacement = self.velocity * PLAYER_SPEED * dt;
        self.position += displacement;
        self.position.y = self.position.y.min(5.).max(1.);
    }
}

impl Default for Dave {
    fn default() -> Self {
        Self {
            position: [0., 2., 0.].into(),
            velocity: Default::default(),
        }
    }
}

fn get_grid() -> Vec<Mesh> {
    let plane_rotation = Quat::from_rotation_x(TAU / 4.0); // 90 degrees
    let mut meshes = Vec::new();

    let grid_size = 8;
    let square_size = 1.0;

    for row in 0..grid_size {
        for column in 0..grid_size {
            let x = (column as f32) * square_size - (grid_size as f32 / 2.0);
            let y = (row as f32) * square_size - (grid_size as f32 / 2.0);
            let colour = if column == 0 || row == 0 {
                [0.5, 0.3, 0.1]
            } else {
                [0., 0.8, 0.0]
            };

            meshes.push(Mesh {
                geometry: Geometry::Plane,
                transform: Affine3A::from_rotation_translation(plane_rotation, [x, 0., y].into()),
                colour: Some(colour.into()),
                ..Default::default()
            })
        }
    }

    meshes
}

pub fn dave(game: &mut Game) {
    let sphere_rotation = Quat::from_rotation_y(TAU / 8.0);
    let dt = game.time.delta();

    game.dave.update(dt, &game.input);
    game.meshes.push(Mesh {
        geometry: Geometry::Cube,
        transform: Affine3A::from_rotation_translation(sphere_rotation, game.dave.position),
        colour: Some([0., 0., 0.9].into()),
        ..Default::default()
    });
}

#[no_mangle]
pub fn tick(game: &mut Game) {
    while game.time.start_update() {
        game.meshes = get_grid();
        dave(game);
    }
}
