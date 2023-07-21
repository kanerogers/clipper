pub use glam;
pub use rand;

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

#[derive(Clone, Default, Debug, Copy)]
pub struct Camera {
    pub position: glam::Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub focus_point: glam::Vec3,
    pub target: glam::Vec3,
    pub desired_distance: f32,
    pub start_distance: f32,
}

impl Camera {
    pub fn matrix(&self) -> glam::Affine3A {
        self.transform().inverse()
    }

    pub fn transform(&self) -> glam::Affine3A {
        let rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.);
        glam::Affine3A::from_rotation_translation(rotation, self.position)
    }
}
