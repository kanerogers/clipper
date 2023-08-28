use common::hecs;
use components::{
    Inventory, Job, JobState, PlaceOfWork, Resource, ResourceDestination, Storage, Task, Transform,
    Viking,
};

use crate::Game;

pub fn viking_work_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    for (_, (viking, transform, job)) in world.query::<(&mut Viking, &Transform, &mut Job)>().iter()
    {
        let current_position = transform.position;
        let place_of_work = world.get::<&PlaceOfWork>(job.place_of_work).unwrap();
        match &mut job.state {
            JobState::GoingToPlaceOfWork => {
                let place_of_work_position = game.position_of(job.place_of_work);
                if place_of_work_position.distance(current_position) <= 2.0 {
                    job.state = JobState::Working(0.);
                }
            }
            JobState::Working(work_time_elapsed) => {
                *work_time_elapsed += dt;
                let mut work_inventory = world.get::<&mut Inventory>(job.place_of_work).unwrap();
                if *work_time_elapsed < place_of_work.task.work_duration() {
                    continue;
                }

                if let Some(resource_destination) = work_complete(
                    place_of_work.task,
                    &mut work_inventory,
                    &mut viking.inventory,
                ) {
                    let drop_off_destination =
                        find_resource_destination(world, resource_destination);
                    job.state = JobState::DroppingOffResource(drop_off_destination);
                } else {
                    *work_time_elapsed = 0.; // try again?
                }
            }
            JobState::DroppingOffResource(destination) => {
                if game.position_of(*destination).distance(current_position) <= 2. {
                    let mut destination_inventory =
                        world.get::<&mut Inventory>(*destination).unwrap();
                    let resource = place_of_work.task.resource();
                    if let Some(amount) = viking.inventory.take(1, &resource) {
                        destination_inventory.add(resource, amount);
                    }
                    job.state = JobState::GoingToPlaceOfWork;
                }
            }
        }
    }
}

fn find_resource_destination(
    world: &hecs::World,
    resource_destination: ResourceDestination,
) -> hecs::Entity {
    match resource_destination {
        ResourceDestination::PlaceOfWork(place_type) => {
            world
                .query::<&PlaceOfWork>()
                .iter()
                .find(|r| r.1.place_type == place_type)
                .unwrap()
                .0
        }
        ResourceDestination::Storage => world.query::<&Storage>().iter().next().unwrap().0,
    }
}

fn work_complete(
    task: Task,
    work_inventory: &mut Inventory,
    viking_inventory: &mut Inventory,
) -> Option<ResourceDestination> {
    let producing_resource = task.resource();
    if let Some(amount_produced) = match task {
        Task::Gather => work_inventory.take(1, &producing_resource),
        Task::Smelt => work_inventory.take(1, &Resource::RawIron),
        Task::MakePaperclips => work_inventory.take(1, &Resource::Iron),
    } {
        let destination = producing_resource.destination();
        viking_inventory.add(producing_resource, amount_produced);
        Some(destination)
    } else {
        println!("Unable to complete task!");
        None
    }
}
