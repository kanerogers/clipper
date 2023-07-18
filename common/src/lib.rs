pub use glam;

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub index_offset: u32,
    pub index_count: u32,
    pub texture_id: u32,
    pub transform: glam::Affine3A,
    pub colour: Option<glam::Vec3>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            index_offset: Default::default(),
            index_count: Default::default(),
            texture_id: u32::MAX,
            transform: Default::default(),
            colour: Default::default(),
        }
    }
}
