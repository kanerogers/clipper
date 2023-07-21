use common::{
    glam::{Affine3A, Vec3},
    Geometry, Mesh,
};

use crate::{Input, Keys, PLAYER_SPEED};

#[derive(Clone, Debug)]
pub struct Dave {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mesh: Mesh,
}

impl Dave {
    pub fn update(&mut self, dt: f32, input: &Input, camera_transform: &Affine3A) {
        let input_movement = Vec3::new(
            input.keyboard_state.as_axis(Keys::A, Keys::D),
            input.keyboard_state.as_axis(Keys::C, Keys::Space),
            input.keyboard_state.as_axis(Keys::W, Keys::S),
        )
        .normalize();

        // Camera relative controls
        let mut forward = camera_transform.transform_vector3(Vec3::Z);
        forward.y = 0.;
        forward = forward.normalize();

        let mut right = camera_transform.transform_vector3(Vec3::X);
        right.y = 0.;
        right = right.normalize();

        let mut movement = forward * input_movement.z + right * input_movement.x;
        movement = movement.normalize_or_zero();
        movement.y = input_movement.y;
        movement = movement.normalize_or_zero();

        self.velocity = self.velocity.lerp(movement, 0.1);

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
