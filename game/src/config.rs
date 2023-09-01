use common::glam::Vec3;

pub const FREE_COLOUR: Vec3 = Vec3::new(1., 0., 0.);
pub const FOLLOWING_COLOUR: Vec3 = Vec3::new(0., 0., 1.);
// pub const WORKING_COLOUR: Vec3 = Vec3::new(0., 0.85, 0.);
pub const BRAINWASH_DISTANCE_THRESHOLD: f32 = 5.0;
pub const BRAINWASH_TIME: f32 = 1.;
pub const MAX_HEALTH: usize = 100;
pub const MAX_ENERGY: usize = 100;
pub const ENERGY_REGEN_TIME: f32 = 2.0;
pub const ENERGY_DRAIN_TIME: f32 = 0.1;
pub const VIKING_MOVE_SPEED: f32 = 4.0;
pub const VIKING_ATTACK_COOLDOWN_TIME: f32 = 1.;
pub const VIKING_ATTACK_DAMAGE: usize = 5;
pub const COMBAT_RANGE: f32 = 1.;
pub const HEALTH_REGEN_RATE: f32 = 0.01;
pub const BUILDING_TRANSPARENCY: f32 = 0.5;
pub const CONSTRUCTION_TIME: f32 = 10.;
