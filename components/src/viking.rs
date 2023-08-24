use std::{time::Instant, fmt::Display};

use common::{
    glam::Vec3, hecs::{self},
};

use crate::Beacon;

use super::{Transform, PlaceOfWork, Inventory, ResourceDestination, Storage, Task, Resource};


#[derive(Clone, Debug)]
pub struct Viking {
    pub state: BrainwashState,
    pub last_update: Instant,
    pub inventory: Inventory,
    pub place_of_work: Option<hecs::Entity>,
    pub intelligence: usize,
    pub strength: usize,
    pub stamina: usize,
}

impl Default for Viking {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            state: BrainwashState::Free,
            inventory: Default::default(),
            place_of_work: None,
            intelligence: 0,
            strength: 0,
            stamina: 0,
        }
    }
}


impl Viking {
    pub fn new(intelligence: usize, strength: usize, stamina: usize) -> Self { 
        Self { 
            intelligence,
            strength,
            stamina,
            ..Default::default() 
        } 
    }

    pub fn update_state(&mut self, current_position: Vec3, world: &hecs::World, dt: f32, me: hecs::Entity) {
        self.state.update(world, dt, current_position, me, self.place_of_work.clone(), &mut self.inventory)
    }

    pub fn assign_place_of_work(&mut self, place_of_work_entity: hecs::Entity) {
        self.place_of_work = Some(place_of_work_entity);
        self.state = BrainwashState::GoingToPlaceOfWork;
    }

    pub fn unassign_work(&mut self) {
        self.place_of_work = None;
        self.state = BrainwashState::AwaitingAssignment;
    }
}


#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum BrainwashState {
    Free,
    BeingBrainwashed(f32),
    Following,
    BecomingWorker(f32),
    AwaitingAssignment,
    GoingToPlaceOfWork,
    Working(f32),
    DroppingOffResource(hecs::Entity),
}

impl Display for BrainwashState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrainwashState::Free => f.write_str("Free"),
            BrainwashState::BeingBrainwashed(a) => f.write_fmt(format_args!("Being brainwashed - {}%", percentage(*a, BRAINWASH_TIME))),
            BrainwashState::Following => f.write_str("Following"),
            BrainwashState::BecomingWorker(a) => f.write_fmt(format_args!("Becoming worker - {}%", percentage(*a, 5.))),
            BrainwashState::AwaitingAssignment => f.write_str("Idle"),
            BrainwashState::GoingToPlaceOfWork => f.write_str("Going to place of work"),
            BrainwashState::Working(a) => f.write_fmt(format_args!("Working - {a:.2}")),
            BrainwashState::DroppingOffResource(_) => f.write_str("Dropping off resource"),
        }
    }
}

fn percentage(val: f32, max: f32) -> usize {
    ((val / max) * 100.) as _
}

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum WorkerState {

}

const WORK_TIME: f32 = 5.;
const BEACON_TAKEOVER_THRESHOLD: f32 = 10.;
pub const BRAINWASH_TIME: f32 = 1.;

impl BrainwashState {
    pub fn update(
        &mut self,
        world: &hecs::World,
        dt: f32,
        current_position: Vec3,
        me: hecs::Entity,
        place_of_work_entity: Option<hecs::Entity>,
        inventory: &mut Inventory,
    ) {
        let nearest_beacon = find_nearest_beacon(&current_position, world);
        match self {
            // handled by brainwashing system
            BrainwashState::Free | BrainwashState::BeingBrainwashed(_) => {},
            BrainwashState::Following => {
                let Some(nearest_beacon_entity) = nearest_beacon else { return };
                let beacon_position = world.get::<&Transform>(nearest_beacon_entity).unwrap().position;
                if beacon_position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *self = BrainwashState::BecomingWorker(0.);
                }
            }
            BrainwashState::BecomingWorker(brainwash_time_elapsed) => {
                let Some(nearest_beacon_entity) = nearest_beacon else { 
                    *self = BrainwashState::Free; 
                    return 
                };
                let beacon_position = world.get::<&Transform>(nearest_beacon_entity).unwrap().position;
                let mut beacon = world.get::<&mut Beacon>(nearest_beacon_entity).unwrap();

                if beacon_position.distance(current_position) <= BEACON_TAKEOVER_THRESHOLD {
                    *brainwash_time_elapsed += dt;
                }

                if *brainwash_time_elapsed >= BRAINWASH_TIME {
                    beacon.workers.insert(me);
                    *self = BrainwashState::AwaitingAssignment;
                }
            }
            BrainwashState::AwaitingAssignment => {}
            BrainwashState::GoingToPlaceOfWork => {
                let place_of_work_position = world.get::<&Transform>(place_of_work_entity.unwrap()).unwrap().position;
                if place_of_work_position.distance(current_position) <= 2.0 {
                    *self = BrainwashState::Working(0.);
                }
            }
            BrainwashState::DroppingOffResource(destination_entity) => {
                let destination_position = world.get::<&Transform>(*destination_entity).unwrap().position;
                let resource = world.get::<&mut PlaceOfWork>(place_of_work_entity.unwrap()).unwrap().task.resource();
                if destination_position.distance(current_position) <= 2.0 {
                    let mut destination_inventory = world.get::<&mut Inventory>(*destination_entity).unwrap();
                    if let Some(amount) = inventory.take(1, &resource) {
                        destination_inventory.add(resource, amount);
                    }
                    *self = BrainwashState::GoingToPlaceOfWork;
                }
            }
            BrainwashState::Working(work_time_elapsed) => {
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
                        *self = BrainwashState::DroppingOffResource(destination);
                        inventory.add(producing_resource, amount_produced);
                    } else {
                        println!("Unable to complete task!");
                        *self = BrainwashState::AwaitingAssignment;
                    }
                    return;
                }

                *work_time_elapsed += dt;
            }
        }
    }
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