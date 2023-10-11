use std::collections::VecDeque;

use common::{
    hecs, log, GUICommand, GUIState, PlaceOfWorkInfo, SelectedItemInfo, VikingInfo,
    BUILDING_TYPE_FACTORY, BUILDING_TYPE_FORGE, BUILDING_TYPE_HOUSE,
};
use components::{
    BrainwashState, Building, BuildingGhost, Collider, ConstructionSite, Dave, GLTFAsset, Health,
    HumanNeeds, Inventory, Job, JobState, MaterialOverrides, PlaceOfWork, Resource, RestState,
    Selected, Storage, Task, Transform, Viking, WorkplaceType,
};

use crate::{
    config::{BUILDING_TRANSPARENCY, MAX_ENERGY, MAX_HEALTH},
    Game,
};

pub fn process_gui_command_queue(
    game: &mut Game,
    command_queue: &mut VecDeque<GUICommand>,
) -> bool {
    let world = &game.world;
    let mut command_buffer = hecs::CommandBuffer::new();
    let mut need_restart = false;

    for command in command_queue.drain(..) {
        match command {
            GUICommand::SetWorkerCount(place_of_work_entity, desired_worker_count) => {
                set_worker_count(
                    world,
                    place_of_work_entity,
                    desired_worker_count,
                    &mut command_buffer,
                );
            }
            GUICommand::Liquify(entity) => command_buffer.despawn(entity),
            GUICommand::Restart => {
                need_restart = true;
            }
            GUICommand::ConstructBuilding(building_name) => {
                show_ghost_building(building_name, world, &mut command_buffer);
            }
        }
    }

    game.run_command_buffer(command_buffer);

    if need_restart {
        *game = crate::init::init_game();
    }

    need_restart
}

pub fn update_gui_state(game: &mut Game, gui_state: &mut GUIState) {
    gui_state.game_over = game.game_over;
    gui_state.idle_workers = game
        .world
        .query_mut::<&Viking>()
        .without::<&Job>()
        .into_iter()
        .filter(|(_, h)| h.brainwash_state == BrainwashState::Brainwashed)
        .count();

    gui_state.paperclips = game
        .world
        .query::<&Inventory>()
        .with::<&Storage>()
        .iter()
        .map(|(_, i)| i.amount_of(Resource::Paperclip))
        .sum();

    {
        let dave = game.world.get::<&Dave>(game.dave).unwrap();
        let health = game.world.get::<&Health>(game.dave).unwrap();
        gui_state.bars.health_percentage = health.value as f32 / MAX_HEALTH as f32;
        gui_state.bars.energy_percentage = dave.energy as f32 / MAX_ENERGY as f32;
    }

    gui_state.clock = format!("{}", game.clock);
    gui_state.clock_description = if game.clock.is_work_time() {
        "Work Time".to_string()
    } else {
        "Rest Time".to_string()
    };
    gui_state.total_deaths = game.total_deaths;

    if let Some((entity, (viking, inventory, job, needs, rest_state))) = game
        .world
        .query::<(&Viking, &Inventory, Option<&Job>, &HumanNeeds, &RestState)>()
        .with::<&Selected>()
        .iter()
        .next()
    {
        let place_of_work = job.map(|j| {
            game.world
                .get::<&PlaceOfWork>(j.place_of_work)
                .unwrap()
                .place_type
        });
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::Viking(VikingInfo {
                inventory: format!("{:?}", inventory),
                name: "Boris".into(),
                state: format!("{}", viking.brainwash_state),
                place_of_work: format!("{place_of_work:?}"),
                intelligence: viking.intelligence,
                strength: viking.strength,
                stamina: viking.stamina,
                needs: format!("{needs:?}"),
                rest_state: format!("{rest_state:?}"),
            }),
        ));
        return;
    }

    if let Some((entity, (place_of_work, inventory))) = game
        .world
        .query_mut::<(&PlaceOfWork, &Inventory)>()
        .with::<&Selected>()
        .into_iter()
        .next()
    {
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::PlaceOfWork(PlaceOfWorkInfo {
                name: format!("{:?}", place_of_work.place_type),
                task: format!("{:?}", place_of_work.task),
                workers: place_of_work.workers.len(),
                max_workers: place_of_work.worker_capacity,
                stock: format!("{inventory:?}"),
            }),
        ));
        return;
    }

    if let Some((entity, (_, inventory))) = game
        .world
        .query_mut::<(&Storage, &Inventory)>()
        .with::<&Selected>()
        .into_iter()
        .next()
    {
        gui_state.selected_item = Some((
            entity,
            SelectedItemInfo::PlaceOfWork(PlaceOfWorkInfo {
                name: "Storage".into(),
                task: "Storing things".into(),
                workers: 0,
                max_workers: 0,
                stock: format!("{inventory:?}"),
            }),
        ));
        return;
    }

    // nothing was selected!
    gui_state.selected_item = None;
}

fn set_worker_count(
    world: &hecs::World,
    place_of_work_entity: hecs::Entity,
    desired_worker_count: usize,
    command_buffer: &mut hecs::CommandBuffer,
) {
    let mut place_of_work = world.get::<&mut PlaceOfWork>(place_of_work_entity).unwrap();
    let current_workers = place_of_work.workers.len();
    if desired_worker_count > current_workers {
        if let Some(worker_entity) = find_available_worker(world) {
            place_of_work.workers.push_front(worker_entity);
            let mut job = Job::new(place_of_work_entity);

            // TODO: this is inelegant
            if place_of_work.task == Task::Construction {
                let storage = world.query::<&Storage>().iter().next().unwrap().0;
                let construction_site = world
                    .get::<&ConstructionSite>(place_of_work_entity)
                    .unwrap();
                job.state =
                    JobState::FetchingResource(construction_site.resources_required().1, storage);
            }
            command_buffer.insert_one(worker_entity, job);
        }
    } else {
        if let Some(worker_entity) = place_of_work.workers.pop_back() {
            command_buffer.remove_one::<Job>(worker_entity);
        }
    }
}

fn show_ghost_building(
    building_name: &str,
    world: &hecs::World,
    command_buffer: &mut hecs::CommandBuffer,
) {
    let (building_type, asset_name) = match building_name {
        BUILDING_TYPE_FACTORY => (Building::PlaceOfWork(WorkplaceType::Factory), "factory.glb"),
        BUILDING_TYPE_FORGE => (Building::PlaceOfWork(WorkplaceType::Forge), "forge.glb"),
        BUILDING_TYPE_HOUSE => (Building::House, "house.glb"),
        _ => {
            log::error!("Attempted to build unknown building type {building_name}");
            return;
        }
    };

    // Remove any existing ghosts
    for (entity, _) in world.query::<()>().with::<&BuildingGhost>().iter() {
        command_buffer.despawn(entity);
    }

    command_buffer.spawn((
        BuildingGhost::new(building_type),
        GLTFAsset::new(asset_name),
        Transform::default(),
        Collider::default(),
        MaterialOverrides {
            base_colour_factor: [1., 1., 1., BUILDING_TRANSPARENCY].into(),
        },
    ));
}

fn find_available_worker(world: &hecs::World) -> Option<hecs::Entity> {
    let mut query = world.query::<&Viking>();
    for (entity, viking) in query.iter() {
        if viking.brainwash_state == BrainwashState::Brainwashed {
            return Some(entity);
        }
    }

    None
}
