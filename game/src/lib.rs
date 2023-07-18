use common::{
    glam::{self, Affine3A},
    Mesh,
};
use std::f32::consts::TAU;

#[derive(Clone, Debug, Default)]
pub struct Game {
    pub meshes: Vec<Mesh>,
}

fn get_grid() -> Vec<Mesh> {
    let plane_rotation = glam::Quat::from_rotation_x(TAU / 4.0); // 90 degrees
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
                index_offset: 0,
                index_count: 6,
                transform: Affine3A::from_rotation_translation(plane_rotation, [x, 0., y].into()),
                colour: Some(colour.into()),
                ..Default::default()
            })
        }
    }

    meshes
}

#[no_mangle]
pub fn tick(game: &mut Game) {
    game.meshes = get_grid();
}
