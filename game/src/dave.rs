use common::{
    glam::{Affine3A, Vec3},
    Geometry, Mesh,
};

use crate::{Input, PLAYER_SPEED};

#[derive(Clone, Debug)]
pub struct Dave {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mesh: Mesh,
}

impl Dave {
    pub fn update(&mut self, dt: f32, input: &Input, camera_transform: &Affine3A) {
        // Camera relative controls
        let mut forward = camera_transform.transform_vector3(Vec3::Z);
        forward.y = 0.;
        forward = forward.normalize();

        let mut right = camera_transform.transform_vector3(Vec3::X);
        right.y = 0.;
        right = right.normalize();

        let mut movement = forward * input.movement.z + right * input.movement.x;
        movement.y = input.movement.y;
        movement = movement.normalize();
        self.velocity = if !movement.is_nan() {
            movement
        } else {
            Vec3::ZERO
        };

        // Velocity, baby!
        let displacement = self.velocity * PLAYER_SPEED * dt;
        self.position += displacement;
        self.position.y = self.position.y.min(5.).max(1.);
        self.mesh.transform = Affine3A::from_translation(self.position);
    }
}

impl Default for Dave {
    fn default() -> Self {
        let position = [0., 2., 0.].into();
        Self {
            position,
            velocity: Default::default(),
            mesh: Mesh {
                geometry: Geometry::Sphere,
                transform: Affine3A::from_translation(position),
                colour: Some([0., 0., 0.9].into()),
                ..Default::default()
            },
        }
    }
}
