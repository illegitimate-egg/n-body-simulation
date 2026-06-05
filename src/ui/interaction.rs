use macroquad::{
    color::Color,
    input::{MouseButton, is_mouse_button_down, mouse_position},
    math::{Circle, Vec2},
    shapes::draw_circle,
};

use crate::{
    MouseStatus,
    state::{Mode, State},
    ui::context_menu::CTXMenu,
};

pub fn handle_interaction(state: &mut State) {
    if is_mouse_button_down(MouseButton::Left) {
        match state.mouse_state {
            MouseStatus::Released => {
                if let Some(ctx) = state.ctx_menu
                    && !(ctx.interaction_rect.min.x < mouse_position().0
                        && ctx.interaction_rect.max.x + ctx.interaction_rect.min.x
                            > mouse_position().0
                        && ctx.interaction_rect.min.y < mouse_position().1
                        && ctx.interaction_rect.max.y + ctx.interaction_rect.min.y
                            > mouse_position().1)
                {
                    state.ctx_menu = None;
                }

                let mut found_index = None;
                {
                    let objects = state.objects.read().unwrap();
                    for i in 0..objects.len() {
                        let radius = Circle::new(
                            objects.position_x[i],
                            objects.position_y[i],
                            5.0 / 1000.0, // 1000 is the zoom factor
                        );
                        if radius.contains(
                            &state
                                .camera_controller
                                .camera
                                .screen_to_world(mouse_position().into()),
                        ) {
                            found_index = Some(i);
                            break;
                        }
                    }
                }

                state.mouse_state = match found_index {
                    Some(idx) => MouseStatus::Dragging(idx),
                    None => MouseStatus::Held,
                };
            }
            MouseStatus::Creating => {
                let position = state
                    .camera_controller
                    .camera
                    .screen_to_world(mouse_position().into());
                state.objects.write().unwrap().insert_object(
                    position,
                    Vec2::new(0.0, 0.0),
                    1000000.0,
                );

                state.mouse_state = MouseStatus::Released;
                state.prediction_dirty = true;
            }
            _ => {}
        }
    } else {
        if state.mouse_state == MouseStatus::CreatingStart {
            state.mouse_state = MouseStatus::Creating;
        } else if state.mouse_state == MouseStatus::Creating {
            let world_mouse = state
                .camera_controller
                .camera
                .screen_to_world(mouse_position().into());
            draw_circle(
                world_mouse.x,
                world_mouse.y,
                5.0 / 1000.0,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.5,
                },
            );
        } else {
            state.mouse_state = MouseStatus::Released;
        }
    }

    match state.mode {
        Mode::Simulating => {}
        Mode::Paused => {
            if let MouseStatus::Dragging(index) = state.mouse_state {
                let position = state
                    .camera_controller
                    .camera
                    .screen_to_world(mouse_position().into());

                {
                    let mut objects = state.objects.write().unwrap();
                    objects.position_x[index] = position.x;
                    objects.position_y[index] = position.y;
                }
                state.prediction_dirty = true;
            }
        }
    }

    // Open ctx menu
    // TODO: Performance Gainz by evaluating circle around the mouse
    if is_mouse_button_down(MouseButton::Right) {
        let mut found_index = None;
        let objects = state.objects.read().unwrap();
        for i in 0..objects.len() {
            let radius = Circle::new(objects.position_x[i], objects.position_y[i], 5.0 / 1000.0);
            if radius.contains(
                &state
                    .camera_controller
                    .camera
                    .screen_to_world(mouse_position().into()),
            ) {
                found_index = Some(i);
                break;
            } else {
                state.ctx_menu = None;
            }
        }

        if let Some(idx) = found_index {
            state.ctx_menu = Some(CTXMenu {
                object: idx,
                position: egui::Pos2::new(mouse_position().0, mouse_position().1),
                interaction_rect: egui::Rect {
                    min: egui::Pos2::new(0.0, 0.0),
                    max: egui::Pos2::new(0.0, 0.0),
                },
            });
        }
    }
}
