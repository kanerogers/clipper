use crate::{ClickState, Game};
use common::hecs;
use components::{Info, Selected};

pub fn click_system(game: &mut Game) {
    let mouse_state = &mut game.input.mouse_state;
    let mut command_buffer = hecs::CommandBuffer::new();
    let mut entity_was_selected = false;
    let mut click_missed = false;
    if let Some(mouse_position) = mouse_state.position {
        if mouse_state.left_click_state == ClickState::JustReleased {
            println!("Clicked at {mouse_position}");
            let ray = game.camera.create_ray(mouse_position);
            println!("Casting ray: {ray:?}");

            game.last_ray = Some(ray);

            if let Some(entity) = game.physics_context.cast_ray(&ray) {
                if let Ok(info) = game.world.get::<&Info>(entity) {
                    command_buffer.insert_one(entity, Selected);
                    entity_was_selected = true;
                    println!("You clicked on {info:?}");
                }
            } else {
                click_missed = true;
            }
        }
    };

    if entity_was_selected || click_missed {
        update_selected_entity(&mut game.world, &mut command_buffer);
    }
}

fn update_selected_entity(world: &mut hecs::World, command_buffer: &mut hecs::CommandBuffer) {
    // first, remove the `Selected` component from any entity that already has it
    // THERE CAN ONLY BE ONE
    for (entity, _) in world.query::<()>().with::<&Selected>().iter() {
        command_buffer.remove_one::<Selected>(entity);
    }

    // now update the world; this command buffer already has the command to insert
    // the `Selected` component, if one was selected
    command_buffer.run_on(world);
}
