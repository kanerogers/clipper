use std::ops::DerefMut;

use common::{hecs, log};
use components::{
    ConstructionSite, House, HumanNeeds, Inventory, Job, JobState, PlaceOfWork, Resource,
    RestState, Storage, Task, Transform, Viking,
};

use crate::{
    config::{EATING_DURATION, SLEEPING_DURATION, STORAGE_RETRIVAL_DISTANCE},
    Game,
};
pub fn viking_behaviour_system(game: &mut Game) {
    if game.clock.is_work_time() {
        viking_work_system(game);
    } else {
        viking_rest_system(game);
    }
}

fn viking_work_system(game: &mut Game) {
    let world = &game.world;
    let mut command_buffer = game.command_buffer();
    let dt = game.time.delta();
    for (viking_entity, (transform, job, rest_state, inventory)) in world
        .query::<(&Transform, &mut Job, &mut RestState, &mut Inventory)>()
        .with::<&Viking>()
        .iter()
    {
        let current_position = transform.position;
        let place_of_work = world.get::<&PlaceOfWork>(job.place_of_work).unwrap();
        let task = place_of_work.task;

        // It's work time! No resting!
        *rest_state = Default::default();

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

                if attempt_to_complete_job(task, work_inventory.deref_mut(), inventory) {
                    let drop_off_destination = find_resource_destination(world);
                    let resource = task.resource_produced().unwrap();
                    job.state = JobState::DroppingOffResource(resource, drop_off_destination);
                } else {
                    *work_time_elapsed = 0.; // try again?
                }
            }
            JobState::DroppingOffResource(resource, destination) => {
                if game.position_of(*destination).distance(current_position)
                    > STORAGE_RETRIVAL_DISTANCE
                {
                    continue;
                }

                let mut destination_inventory = world.get::<&mut Inventory>(*destination).unwrap();
                if let Some(amount) = inventory.take(1, *resource) {
                    destination_inventory.add(*resource, amount);
                }

                job.state = JobState::GoingToPlaceOfWork;
            }
            JobState::FetchingResource(resource, destination) => {
                if game.position_of(*destination).distance(current_position)
                    > STORAGE_RETRIVAL_DISTANCE
                {
                    continue;
                }

                let mut destination_inventory = world.get::<&mut Inventory>(*destination).unwrap();
                if let Some(amount) = destination_inventory.take(1, *resource) {
                    inventory.add(*resource, amount);
                    job.state = JobState::DroppingOffResource(*resource, job.place_of_work);
                    continue;
                }

                // We can't get the resource we need! Abandon hope.
                log::debug!("Viking failed to fetch resource {resource:?} - removing job {task:?}");
                command_buffer.remove_one::<Job>(viking_entity);
            }
            JobState::Constructing => {
                let mut construction_site = world
                    .get::<&mut ConstructionSite>(job.place_of_work)
                    .unwrap();
                construction_site.construction_progress += dt;
            }
        }
    }
    game.run_command_buffer(command_buffer);
}

fn viking_rest_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();

    for (viking_entity, (transform, needs, rest_state, inventory, job)) in world
        .query::<(
            &Transform,
            &mut HumanNeeds,
            &mut RestState,
            &mut Inventory,
            &mut Job,
        )>()
        .iter()
    {
        // lay down your tools, now it is time to rest
        job.state = JobState::GoingToPlaceOfWork;

        match rest_state {
            RestState::Idle => {
                if needs.hunger > 0 {
                    if inventory.amount_of(Resource::Food) > 0 {
                        *rest_state = RestState::Eating(0.);
                    } else {
                        *rest_state = RestState::GettingFood(game.storage());
                    }
                    continue;
                }

                // Is there an available house?
                if let Some(available_house) = find_available_house(world) {
                    occupy_house(world, available_house, viking_entity);
                    *rest_state = RestState::GoingHome(available_house);
                    continue;
                }

                // Nowhere to sleep!
                *rest_state = RestState::NoHomeAvailable;
            }
            RestState::GettingFood(storage_entity) => {
                if transform
                    .position
                    .distance(game.position_of(*storage_entity))
                    > STORAGE_RETRIVAL_DISTANCE
                {
                    continue;
                }

                // we are at the storage location
                let mut target_inventory = world.get::<&mut Inventory>(*storage_entity).unwrap();
                if target_inventory.take(1, Resource::Food).is_some() {
                    inventory.add(Resource::Food, 1);
                    *rest_state = RestState::Eating(0.);
                } else {
                    // There's no food!
                    *rest_state = RestState::NoFoodAvailable;
                    log::debug!("No food available!");
                }
            }
            RestState::Eating(duration) => {
                *duration += dt;
                if *duration > EATING_DURATION && inventory.take(1, Resource::Food) == Some(1) {
                    needs.hunger = needs.hunger.saturating_sub(1);
                    *rest_state = RestState::Idle;
                }
            }
            RestState::GoingHome(house_entity) => {
                if transform.position.distance(game.position_of(*house_entity)) > 2. {
                    continue;
                }

                // we are at home
                *rest_state = RestState::Sleeping(0.);
            }
            RestState::Sleeping(duration) => {
                *duration += dt;
                if *duration < SLEEPING_DURATION {
                    continue;
                }

                needs.sleep = needs.sleep.saturating_sub(1);
                *duration = 0.;
            }
            RestState::NoFoodAvailable => {
                // Is there an available house?
                if let Some(available_house) = find_available_house(world) {
                    occupy_house(world, available_house, viking_entity);
                    *rest_state = RestState::GoingHome(available_house);
                    continue;
                }

                // Nowhere to sleep!
                log::debug!("No home available!");
                *rest_state = RestState::NoHomeAvailable;
            }
            RestState::NoHomeAvailable => {
                // Nothing to do.
            }
        }
    }
}

fn occupy_house(world: &hecs::World, available_house: hecs::Entity, viking_entity: hecs::Entity) {
    world
        .get::<&mut House>(available_house)
        .unwrap()
        .occupants
        .push(viking_entity);
}

fn find_available_house(world: &hecs::World) -> Option<hecs::Entity> {
    world
        .query::<&mut House>()
        .iter()
        .find_map(|(entity, house)| {
            if house.has_capacity() {
                Some(entity)
            } else {
                None
            }
        })
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
