use renderer::glam;
use std::f32::consts::TAU;

#[derive(Clone, Debug, Default)]
pub struct Game {
    pub meshes: Vec<Mesh>,
}

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub index_offset: u32,
    pub index_count: u32,
    pub texture_id: u32,
    pub transform: glam::Affine3A,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            index_offset: Default::default(),
            index_count: Default::default(),
            texture_id: u32::MAX,
            transform: Default::default(),
        }
    }
}

#[no_mangle]
pub fn tick(game: &mut Game) {
    let plane_rotation = glam::Quat::from_rotation_x(TAU / 4.0); // 90 degrees
    let plane_transform =
        glam::Affine3A::from_rotation_translation(plane_rotation, [-1.0, 0.0, 1.0].into());

    game.meshes = vec![Mesh {
        index_count: 6,
        transform: plane_transform,
        ..Default::default()
    }];
}
