pub use glam;

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub geometry: Geometry,
    pub texture_id: u32,
    pub transform: glam::Affine3A,
    pub colour: Option<glam::Vec3>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            geometry: Geometry::Plane,
            texture_id: u32::MAX,
            transform: Default::default(),
            colour: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Geometry {
    Plane,
    Sphere,
    Cube,
}
