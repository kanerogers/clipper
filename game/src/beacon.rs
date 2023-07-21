use common::{
    glam::{Affine3A, Vec3},
    Mesh,
};

#[derive(Debug, Clone, Default)]
pub struct Beacon {
    pub position: Vec3,
    pub mesh: Mesh,
}

impl Beacon {
    pub fn new(position: Vec3) -> Self {
        let transform = Affine3A::from_scale_rotation_translation(
            [1., 8., 1.].into(),
            Default::default(),
            position,
        );
        Self {
            position,
            mesh: Mesh {
                geometry: common::Geometry::Cube,
                colour: Some([0., 0., 0.].into()),
                transform,
                ..Default::default()
            },
        }
    }
}
