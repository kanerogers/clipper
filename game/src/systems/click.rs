use crate::{components::Info, ClickState, Game};

pub fn click_system(game: &mut Game) {
    let mouse_state = &mut game.input.mouse_state;
    if let Some(mouse_position) = mouse_state.position {
        if mouse_state.left_click_state == ClickState::JustReleased {
            println!("Clicked at {mouse_position}");
            let ray = game.camera.create_ray(mouse_position);
            println!("Casting ray: {ray:?}");

            game.last_ray = Some(ray);

            if let Some(entity) = game.physics_context.cast_ray(&ray) {
                let info = game.world.get::<&Info>(entity).unwrap();
                println!("You clicked on {info:?}");
            }
        }
    };

    reset_mouse_clicks(mouse_state);
}

fn reset_mouse_clicks(mouse_state: &mut crate::MouseState) {
    match mouse_state.left_click_state {
        ClickState::JustReleased => mouse_state.left_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.right_click_state {
        ClickState::JustReleased => mouse_state.right_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.middle_click_state {
        ClickState::JustReleased => mouse_state.middle_click_state = ClickState::Released,
        _ => {}
    };
}
