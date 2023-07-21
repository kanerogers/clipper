use std::time::Instant;

use common::{
    glam::{Affine3A, Vec3},
    rand, Mesh,
};

use crate::Game;

#[derive(Clone, Debug)]
pub struct Human {
    pub mesh: Mesh,
    pub position: Vec3,
    pub velocity: Vec3,
    last_update: Instant,
}

impl Default for Human {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            position: Default::default(),
            velocity: Default::default(),
            last_update: Instant::now(),
        }
    }
}

impl Human {
    pub fn new(position: Vec3) -> Self {
        let mesh = Mesh {
            geometry: common::Geometry::Cube,
            transform: Affine3A::from_translation(position),
            colour: Some([0., 1., 0.].into()),
            ..Default::default()
        };
        Self {
            position,
            mesh,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dt: f32) {
        if Instant::now()
            .duration_since(self.last_update)
            .as_secs_f32()
            > 1.0
        {
            self.last_update = Instant::now();
            self.velocity = random_movement();

            if rand::random() {
                self.velocity = self.velocity.normalize() * 4.;
            } else {
                self.velocity = self.velocity.normalize() * -4.;
            }
        }

        let displacement = self.velocity * dt;
        self.position += displacement;
        self.mesh.transform = Affine3A::from_translation(self.position);
    }
}

pub fn humans(game: &mut Game) {
    let dt = game.time.delta();
    for human in &mut game.humans {
        human.update(dt);
    }
}

fn random_movement() -> Vec3 {
    let x: f32 = rand::random();
    let z: f32 = rand::random();

    [x, 0., z].into()
}
