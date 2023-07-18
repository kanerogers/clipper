use common::{glam, Mesh};
use std::f32::consts::TAU;

#[derive(Clone, Debug, Default)]
pub struct Game {
    pub meshes: Vec<Mesh>,
}

#[no_mangle]
pub fn tick(game: &mut Game) {
    let plane_rotation = glam::Quat::from_rotation_x(TAU / 4.0); // 90 degrees
    let plane_transform =
        glam::Affine3A::from_rotation_translation(plane_rotation, [-1.0, 0.0, 1.0].into());

    game.meshes = vec![Mesh {
        index_count: 6,
        transform: plane_transform,
        colour: Some([1.0, 0., 0.].into()),
        ..Default::default()
    }];
}
