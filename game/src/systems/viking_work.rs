use common::{hecs, log};
use components::{
    ConstructionSite, Inventory, Job, JobState, PlaceOfWork, Storage, Task, Transform, Viking,
};

use crate::Game;

pub fn viking_work_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    for (_, (viking, transform, job)) in world.query::<(&mut Viking, &Transform, &mut Job)>().iter()
    {
        let current_position = transform.position;
        let place_of_work = world.get::<&PlaceOfWork>(job.place_of_work).unwrap();
        let task = place_of_work.task;
        match &mut job.state {
            JobState::GoingToPlaceOfWork => {
                let place_of_work_position = game.position_of(job.place_of_work);
                if place_of_work_position.distance(current_position) <= 2.0 {
                    job.state = match place_of_work.task {
                        Task::Construction => JobState::Constructing,
                        _ => JobState::Working(0.),
                    };
                }
            }
            JobState::Working(work_time_elapsed) => {
                *work_time_elapsed += dt;
                let mut work_inventory = world.get::<&mut Inventory>(job.place_of_work).unwrap();

                // Does this job require a specific kind of resource?
                if let Some(resource_consumed) = task.resource_consumed() {
                    if work_inventory.amount_of(resource_consumed) == 0 {
                        job.state = JobState::FetchingResource(resource_consumed, game.storage());
                        continue;
                    }
                }

                if *work_time_elapsed < place_of_work.task.work_duration() {
                    continue;
                }

                log::info!(
                    "Spent {work_time_elapsed} working, task required {}, dt: {}",
                    place_of_work.task.work_duration(),
                    dt
                );

                if attempt_to_complete_job(task, &mut work_inventory, &mut viking.inventory) {
                    let drop_off_destination = find_resource_destination(world);
                    let resource = task.resource_produced().unwrap();
                    job.state = JobState::DroppingOffResource(resource, drop_off_destination);
                } else {
                    *work_time_elapsed = 0.; // try again?
                }
            }
            JobState::DroppingOffResource(resource, destination) => {
                if game.position_of(*destination).distance(current_position) <= 2. {
                    let mut destination_inventory =
                        world.get::<&mut Inventory>(*destination).unwrap();

                    if let Some(amount) = viking.inventory.take(1, *resource) {
                        destination_inventory.add(*resource, amount);
                    }

                    job.state = JobState::GoingToPlaceOfWork;
                }
            }
            JobState::FetchingResource(resource, destination) => {
                if game.position_of(*destination).distance(current_position) <= 2. {
                    let mut destination_inventory =
                        world.get::<&mut Inventory>(*destination).unwrap();
                    if let Some(amount) = destination_inventory.take(1, *resource) {
                        viking.inventory.add(*resource, amount);
                        job.state = JobState::DroppingOffResource(*resource, job.place_of_work);
                    }
                }
            }
            JobState::Constructing => {
                let mut construction_site = world
                    .get::<&mut ConstructionSite>(job.place_of_work)
                    .unwrap();
                construction_site.construction_progress += dt;
            }
        }
    }
}

fn find_resource_destination(world: &hecs::World) -> hecs::Entity {
    world.query::<&Storage>().iter().next().unwrap().0
}

fn attempt_to_complete_job(
    task: Task,
    work_inventory: &mut Inventory,
    viking_inventory: &mut Inventory,
) -> bool {
    let Some(resource_produced) = task.resource_produced() else { return false };
    if let Some(resource_consumed) = task.resource_consumed() {
        if work_inventory.take(1, resource_consumed).is_none() {
            log::error!("Unable to complete job!");
            return false;
        }
    }

    viking_inventory.add(resource_produced, 1);
    true
}
