pub use bitflags;
pub use glam;
pub use hecs;
pub use log;
pub use rand;
pub use rapier3d;
pub use winit;
pub use yakui;

use glam::Vec3;

#[derive(Clone, Debug, Copy)]
pub struct Mesh {
    pub geometry: Geometry,
    pub texture_id: u32,
    pub transform: glam::Affine3A,
    pub colour: Option<Vec3>,
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

#[derive(Clone, Debug, Copy)]
pub struct Material {
    pub colour: glam::Vec3,
    pub texture_id: u32,
}

impl Material {
    pub fn from_colour(colour: glam::Vec3) -> Self {
        Self {
            colour,
            ..Default::default()
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            colour: Vec3::ONE,
            texture_id: u32::MAX,
        }
    }
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

#[derive(Debug, Clone, Default)]
pub struct GUIState {
    pub paperclips: usize,
    pub workers: usize,
}

pub trait Renderer {
    fn init(window: winit::window::Window) -> Self;
    fn render(&mut self, meshes: &[Mesh], camera: Camera, yak: &mut yakui::Yakui);
    fn resized(&mut self, size: winit::dpi::PhysicalSize<u32>);
    fn cleanup(&mut self);
}
