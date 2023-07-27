use std::time::Instant;

use common::{
    glam::Vec3,
    rand, hecs::{RefMut, self},
};

use crate::beacon::Beacon;

use super::Transform;

#[derive(Clone, Debug)]
pub struct Human {
    pub state: State,
    last_update: Instant,
}

impl Default for Human {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            state: State::Free,
        }
    }
}

impl Human {
    pub fn update_velocity(&mut self, velocity: &mut Vec3, position: Vec3, dave_position: Vec3) {
        match self.state {
            State::Free | State::BeingBrainwashed(_) => {
                if Instant::now()
                    .duration_since(self.last_update)
                    .as_secs_f32()
                    > 1.0
                {
                    self.last_update = Instant::now();
                    *velocity = random_movement();

                    if rand::random() {
                        *velocity = velocity.normalize() * 4.;
                    } else {
                        *velocity = velocity.normalize() * -4.;
                    }
                }
            }
            State::Following | State::BecomingWorker(_) => {
                *velocity = (dave_position - position).normalize() * 2.;
            }
            State::Working => {
                *velocity = Default::default();
            }
        }
    }

}


#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum State {
    Free,
    BeingBrainwashed(f32),
    Following,
    BecomingWorker(f32),
    Working,
}

const FREE_COLOUR: Vec3 = Vec3::new(1., 0., 0.);
const FOLLOWING_COLOUR: Vec3 = Vec3::new(0., 0., 1.);
const WORKING_COLOUR: Vec3 = Vec3::new(0., 1., 0.);
const BRAINWASH_TIME: f32 = 5.;
const BRAINWASH_DISTANCE_THRESHOLD: f32 = 5.0;
const BEACON_TAKEOVER_THRESHOLD: f32 = 10.;

impl State {
    pub fn update(
        &mut self,
        world: &hecs::World,
        dt: f32,
        current_position: Vec3,
        dave_position: Vec3,
        nearest_beacon: Option<hecs::Entity>,
        me: hecs::Entity,
    ) {
        let distance_to_dave = current_position.distance(dave_position);
        let within_brainwash_threshold = distance_to_dave <= BRAINWASH_DISTANCE_THRESHOLD;
        match self {
            State::Free => {
                if within_brainwash_threshold {
                    *self = State::BeingBrainwashed(0.);
                }
            }
            State::BeingBrainwashed(ref mut amount) => {
                if within_brainwash_threshold {
                    *amount += dt;
                } else {
                    *self = State::Free;
                    return;
                }
                if *amount >= BRAINWASH_TIME {
                    *self = State::Following;
                }
            }
            State::Following => {
                if !within_brainwash_threshold {
                    *self = State::Free;
                }

                let Some(nearest_beacon_entity) = nearest_beacon else { return };
                let beacon_position = world.get::<&Transform>(nearest_beacon_entity).unwrap().position;
                if beacon_position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *self = State::BecomingWorker(0.);
                }
            }
            State::BecomingWorker(ref mut amount) => {
                let Some(nearest_beacon_entity) = nearest_beacon else { 
                    *self = State::Free; 
                    return 
                };
                let beacon_position = world.get::<&Transform>(nearest_beacon_entity).unwrap().position;
                let mut beacon = world.get::<&mut Beacon>(nearest_beacon_entity).unwrap();

                if beacon_position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *amount += dt;
                }

                if *amount >= BRAINWASH_TIME {
                    beacon.workers.insert(me);
                    *self = State::Working;
                }
            }
            State::Working => {}
        }
    }
    pub fn get_colour(&self) -> Vec3 {
        match self {
            State::Free => FREE_COLOUR,
            State::BeingBrainwashed(amount) => {
                let brainwashed_percentage = *amount as f32 / BRAINWASH_TIME as f32;
                FREE_COLOUR.lerp(FOLLOWING_COLOUR, brainwashed_percentage)
            }
            State::Following => FOLLOWING_COLOUR,
            State::BecomingWorker(amount) => {
                let brainwashed_percentage = *amount as f32 / BRAINWASH_TIME as f32;
                FOLLOWING_COLOUR.lerp(WORKING_COLOUR, brainwashed_percentage)
            }
            State::Working => WORKING_COLOUR,
        }
    }
}



fn random_movement() -> Vec3 {
    let x: f32 = rand::random();
    let z: f32 = rand::random();

    [x, 0., z].into()
}
