use std::time::Instant;

use common::{
    glam::{Affine3A, Vec3},
    rand, Mesh,
};

use crate::{beacon::Beacon, Game};

#[derive(Clone, Debug)]
pub struct Human {
    pub mesh: Mesh,
    pub position: Vec3,
    pub velocity: Vec3,
    pub state: State,
    last_update: Instant,
}

impl Default for Human {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            position: Default::default(),
            velocity: Default::default(),
            last_update: Instant::now(),
            state: State::Free,
        }
    }
}

impl Human {
    pub fn new(position: Vec3) -> Self {
        let mesh = Mesh {
            geometry: common::Geometry::Cube,
            transform: Affine3A::from_translation(position),
            colour: Some([0., 1., 0.].into()),
            ..Default::default()
        };
        Self {
            position,
            mesh,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dt: f32, dave_position: Vec3, nearest_beacon: Option<&mut Beacon>) {
        self.state
            .update(dt, self.position, dave_position, nearest_beacon);
        self.set_velocity(self.state, dave_position);

        let displacement = self.velocity * dt;
        self.position += displacement;
        self.mesh.colour = Some(self.state.get_colour());
        self.mesh.transform = Affine3A::from_translation(self.position);
    }

    fn set_velocity(&mut self, state: State, dave_position: Vec3) {
        match state {
            State::Free | State::BeingBrainwashed(_) => {
                if Instant::now()
                    .duration_since(self.last_update)
                    .as_secs_f32()
                    > 1.0
                {
                    self.last_update = Instant::now();
                    self.velocity = random_movement();

                    if rand::random() {
                        self.velocity = self.velocity.normalize() * 4.;
                    } else {
                        self.velocity = self.velocity.normalize() * -4.;
                    }
                }
            }
            State::Following | State::BecomingWorker(_) => {
                self.velocity = (dave_position - self.position).normalize();
            }
            State::Working => {
                self.velocity = Default::default();
            }
        }
    }
}

#[derive(Clone, Debug, Copy)]
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
        dt: f32,
        current_position: Vec3,
        dave_position: Vec3,
        nearest_beacon: Option<&mut Beacon>,
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

                let Some(nearest_beacon) = nearest_beacon else { return };
                if nearest_beacon.position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *self = State::BecomingWorker(0.);
                }
            }
            State::BecomingWorker(ref mut amount) => {
                let Some(nearest_beacon) = nearest_beacon else { 
                    *self = State::Free; 
                    return 
                };
                let distance_to_beacon = nearest_beacon.position.distance(current_position);
                if distance_to_beacon <= BEACON_TAKEOVER_THRESHOLD {
                    *amount += dt;
                }

                if *amount >= BRAINWASH_TIME {
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

pub fn humans(game: &mut Game) {
    let dt = game.time.delta();
    let dave_position = game.dave.position;
    for human in &mut game.humans {
        let nearest_beacon = find_nearest_beacon(human.position, &mut game.beacons);
        human.update(dt, dave_position, nearest_beacon);
    }
}

fn find_nearest_beacon(position: Vec3, beacons: &mut [Beacon]) -> Option<&mut Beacon> {
    let mut shortest_distance_found = f32::INFINITY;
    let mut nearest_beacon = None;
    for beacon in beacons {
        let distance = position.distance(beacon.position);
        if distance <= shortest_distance_found {
            shortest_distance_found = distance;
            nearest_beacon = Some(beacon);
        }
    }

    nearest_beacon
}

fn random_movement() -> Vec3 {
    let x: f32 = rand::random();
    let z: f32 = rand::random();

    [x, 0., z].into()
}
