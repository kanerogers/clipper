use common::hecs::{self, Or};
use components::{BrainwashState, CombatState, Health, Job, Transform, Viking};

use crate::{
    config::{COMBAT_RANGE, VIKING_ATTACK_COOLDOWN_TIME, VIKING_ATTACK_DAMAGE},
    Game,
};

pub fn combat_system(game: &mut Game) {
    let mut command_buffer = game.command_buffer();
    // Are there any vikings that need to enter combat?
    enter_combat(game, &mut command_buffer);

    // Are there any vikings that need to leave combat?
    leave_combat(game, &mut command_buffer);

    // Now handle all the vikings currently in combat.
    handle_combat(game);

    game.run_command_buffer(command_buffer);
}

fn handle_combat(game: &mut Game) {
    let world = &game.world;
    for (_, (viking, transform, combat_state)) in world
        .query::<(&Viking, &Transform, &mut CombatState)>()
        .iter()
    {
        let target_position = game.position_of(combat_state.target);
        if transform.position.distance(target_position) > COMBAT_RANGE {
            continue;
        }

        let mut target_health = game.get::<Health>(combat_state.target);
        if game.time.elapsed(combat_state.last_attack_time) > VIKING_ATTACK_COOLDOWN_TIME {
            println!("Attacking!");
            target_health.take(VIKING_ATTACK_DAMAGE * viking.strength, game.now());
            combat_state.last_attack_time = Default::default();
        }
    }
}

fn enter_combat(game: &mut Game, command_buffer: &mut hecs::CommandBuffer) {
    let dave = game.dave;
    let world = &game.world;
    for (viking_entity, viking) in world
        .query::<&Viking>()
        .without::<Or<&CombatState, &Job>>()
        .iter()
    {
        let Viking {
            brainwash_state,
            strength,
            ..
        } = viking;
        match brainwash_state {
            components::BrainwashState::BeingBrainwashed(_) if *strength > 1 => {
                println!("Entering combat!");
                command_buffer.insert_one(viking_entity, CombatState::new(dave))
            }
            _ => {}
        }
    }
}

fn leave_combat(game: &mut Game, command_buffer: &mut hecs::CommandBuffer) {
    let world = &game.world;
    for (viking_entity, viking) in world.query::<&Viking>().with::<&CombatState>().iter() {
        match viking.brainwash_state {
            BrainwashState::Brainwashed | BrainwashState::Free => {
                println!("Leaving combat!");
                command_buffer.remove_one::<CombatState>(viking_entity);
            }
            _ => {}
        }
    }
}
