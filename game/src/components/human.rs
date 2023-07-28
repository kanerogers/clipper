use std::time::Instant;

use common::{
    glam::Vec3,
    rand, hecs::{self},
};

use crate::beacon::Beacon;

use super::{Transform, PlaceOfWork, Inventory, ResourceDestination, Storage, Task, Resource};


#[derive(Clone, Debug)]
pub struct Human {
    pub state: State,
    last_update: Instant,
    pub inventory: Inventory,
    pub place_of_work: Option<hecs::Entity>,
}

impl Default for Human {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            state: State::Free,
            inventory: Default::default(),
            place_of_work: None,
        }
    }
}

impl Human {
    pub fn update_velocity(&mut self, velocity: &mut Vec3, current_position: Vec3, dave_position: Vec3, world: &hecs::World) {
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
                *velocity = (dave_position - current_position).normalize() * 2.;
            }
            State::GoingToPlaceOfWork => {
                    let place_of_work_position = world.get::<&Transform>(self.place_of_work.unwrap()).unwrap().position;
                        *velocity = (place_of_work_position - current_position).normalize() * 2.;
            }
            State::DroppingOffResource(destination) => {
                    let destination_position = world.get::<&Transform>(destination).unwrap().position;
                        *velocity = (destination_position - current_position).normalize() * 2.;
            }
            _ => {
                *velocity = Default::default();
            }
        }
    }

    pub fn update_state(&mut self, current_position: Vec3, dave_position: Vec3, world: &hecs::World, dt: f32, me: hecs::Entity) {
        self.state.update(world, dt, current_position, dave_position, me, self.place_of_work.clone(), &mut self.inventory)
    }

    pub fn assign_place_of_work(&mut self, place_of_work_entity: hecs::Entity) {
        self.place_of_work = Some(place_of_work_entity);
        self.state = State::GoingToPlaceOfWork;
    }

    pub fn unassign_work(&mut self) {
        self.place_of_work = None;
        self.state = State::AwaitingAssignment;
    }

}


#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum State {
    Free,
    BeingBrainwashed(f32),
    Following,
    BecomingWorker(f32),
    AwaitingAssignment,
    GoingToPlaceOfWork,
    Working(f32),
    DroppingOffResource(hecs::Entity),
}

const FREE_COLOUR: Vec3 = Vec3::new(1., 0., 0.);
const FOLLOWING_COLOUR: Vec3 = Vec3::new(0., 0., 1.);
const AWAITING_ASSIGNMENT_COLOUR: Vec3 = Vec3::new(1., 0.85, 0.);
const WORKING_COLOUR: Vec3 = Vec3::new(0., 0.85, 0.);
const BRAINWASH_TIME: f32 = 5.;
const WORK_TIME: f32 = 5.;
const BRAINWASH_DISTANCE_THRESHOLD: f32 = 5.0;
const BEACON_TAKEOVER_THRESHOLD: f32 = 10.;

impl State {
    pub fn update(
        &mut self,
        world: &hecs::World,
        dt: f32,
        current_position: Vec3,
        dave_position: Vec3,
        me: hecs::Entity,
        place_of_work_entity: Option<hecs::Entity>,
        inventory: &mut Inventory,
    ) {
        let distance_to_dave = current_position.distance(dave_position);
        let within_brainwash_threshold = distance_to_dave <= BRAINWASH_DISTANCE_THRESHOLD;
        let nearest_beacon = find_nearest_beacon(&current_position, world);
        match self {
            State::Free => {
                if within_brainwash_threshold {
                    *self = State::BeingBrainwashed(0.);
                }
            }
            State::BeingBrainwashed(brainwash_time_elapsed) => {
                if within_brainwash_threshold {
                    *brainwash_time_elapsed += dt;
                } else {
                    *self = State::Free;
                    return;
                }
                if *brainwash_time_elapsed >= BRAINWASH_TIME {
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
            State::BecomingWorker(brainwash_time_elapsed) => {
                let Some(nearest_beacon_entity) = nearest_beacon else { 
                    *self = State::Free; 
                    return 
                };
                let beacon_position = world.get::<&Transform>(nearest_beacon_entity).unwrap().position;
                let mut beacon = world.get::<&mut Beacon>(nearest_beacon_entity).unwrap();

                if beacon_position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *brainwash_time_elapsed += dt;
                }

                if *brainwash_time_elapsed >= BRAINWASH_TIME {
                    beacon.workers.insert(me);
                    *self = State::AwaitingAssignment;
                }
            }
            State::AwaitingAssignment => {}
            State::GoingToPlaceOfWork => {
                let place_of_work_position = world.get::<&Transform>(place_of_work_entity.unwrap()).unwrap().position;
                if place_of_work_position.distance(current_position) <= 2.0 {
                    *self = State::Working(0.);
                }
            }
            State::DroppingOffResource(destination_entity) => {
                let destination_position = world.get::<&Transform>(*destination_entity).unwrap().position;
                let resource = world.get::<&mut PlaceOfWork>(place_of_work_entity.unwrap()).unwrap().task.resource();
                if destination_position.distance(current_position) <= 2.0 {
                    let mut destination_inventory = world.get::<&mut Inventory>(*destination_entity).unwrap();
                    if let Some(amount) = inventory.take(1, &resource) {
                        destination_inventory.add(resource, amount);
                    }
                    *self = State::GoingToPlaceOfWork;
                }
            }
            State::Working(work_time_elapsed) => {
                if *work_time_elapsed >= WORK_TIME {
                    let task = world.get::<&mut PlaceOfWork>(place_of_work_entity.unwrap()).unwrap().task;
                    let producing_resource = task.resource();
                    let mut work_inventory = world.get::<&mut Inventory>(place_of_work_entity.unwrap()).unwrap();
                    if let Some(amount_produced) = match task {
                        Task::Gather => {
                            work_inventory.take(1, &producing_resource)
                        }
                        Task::Smelt => {
                            work_inventory.take(1, &Resource::RawIron)
                        }
                        Task::MakePaperclips => {
                            work_inventory.take(1, &Resource::Iron)
                        }
                    } {
                        let destination = match producing_resource.destination() {
                            ResourceDestination::Storage => {
                                world.query::<()>().with::<&Storage>().iter().next().unwrap().0
                            },
                            ResourceDestination::PlaceOfWork(place) => {
                                world.query::<&PlaceOfWork>().iter().find(|r| r.1.place_type == place).unwrap().0
                            }
                        };
                        *self = State::DroppingOffResource(destination);
                        inventory.add(producing_resource, amount_produced);
                    } else {
                        println!("Unable to complete task!");
                        *self = State::AwaitingAssignment;
                    }
                    return;
                }

                *work_time_elapsed += dt;
            }
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
                FOLLOWING_COLOUR.lerp(AWAITING_ASSIGNMENT_COLOUR, brainwashed_percentage)
            }
            State::AwaitingAssignment => AWAITING_ASSIGNMENT_COLOUR,
            _ => WORKING_COLOUR,
        }
    }
}



fn random_movement() -> Vec3 {
    let x: f32 = rand::random();
    let z: f32 = rand::random();

    [x, 0., z].into()
}

fn find_nearest_beacon<'a>(position: &Vec3, world: &'a hecs::World) -> Option<hecs::Entity> {
    let mut shortest_distance_found = f32::INFINITY;
    let mut nearest_beacon = None;
    for (entity, (transform, _beacon)) in world.query::<(&Transform, &'a mut Beacon)>().iter() {
        let distance = position.distance(transform.position);
        if distance <= shortest_distance_found {
            shortest_distance_found = distance;
            nearest_beacon = Some(entity);
        }
    }

    nearest_beacon
}