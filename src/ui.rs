use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::{
    Mode, MouseStatus, Objects, State,
    phys::{PredictionDirection, Predictor},
};

#[derive(Debug, Clone, Copy)]
pub struct CTXMenu {
    object: usize,
    position: egui::Pos2,
    pub interaction_rect: egui::Rect,
}

pub struct CameraController {
    pub camera: Camera2D,
    dragging: bool,
    last_mouse: Vec2,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            camera: Camera2D {
                zoom: vec2(2.0 / screen_width() * 200.0, -2.0 / screen_height() * 200.0),
                target: vec2(0.0, 0.0),
                ..Default::default()
            },
            dragging: false,
            last_mouse: mouse_position().into(),
        }
    }

    pub fn update(&mut self) {
        let mouse: Vec2 = mouse_position().into();

        // Middle mouse OR Alt + Left Mouse
        let drag_active = is_mouse_button_down(MouseButton::Middle)
            || (is_key_down(KeyCode::LeftAlt) && is_mouse_button_down(MouseButton::Left));

        // Start drag
        if drag_active && !self.dragging {
            self.dragging = true;
            self.last_mouse = mouse;
        }

        // End drag
        if !drag_active {
            self.dragging = false;
        }

        // Pan camera
        if self.dragging {
            let mut delta = mouse - self.last_mouse;

            // Flip y axis movement
            delta.y = -delta.y;

            // Convert screen movement into world movement
            let zoom_scale = vec2(2.0 / screen_width(), 2.0 / screen_height());

            self.camera.target -= delta * zoom_scale / self.camera.zoom.abs();

            self.last_mouse = mouse;
        }
        // Scroll zoom
        let (_, scroll_y) = mouse_wheel();

        if scroll_y != 0.0 {
            // Mouse position in screen space
            let mouse_screen: Vec2 = mouse_position().into();

            // World position BEFORE zoom
            let world_before = self.camera.screen_to_world(mouse_screen);

            // Zoom factor
            let zoom_factor = if scroll_y > 0.0 { 1.1 } else { 0.9 };

            // Apply zoom
            self.camera.zoom *= zoom_factor;

            // Clamp zoom
            self.camera.zoom.x = self.camera.zoom.x.clamp(0.0005, 10.0);

            self.camera.zoom.y = self.camera.zoom.x * -(screen_width() / screen_height()); // Recompute y

            // World position AFTER zoom
            let world_after = self.camera.screen_to_world(mouse_screen);

            // Move camera target so cursor stays fixed on same world point
            self.camera.target += world_before - world_after;
        }
    }
}

pub fn draw_objects(objects: &Objects) {
    for i in 0..objects.len() {
        // Draw it
        draw_circle(
            objects.position_x[i],
            objects.position_y[i],
            5.0 / 1000.0,
            BLACK,
        ); // Draw a thing
    }
}

// Fixed allocations for draw_prediction
pub struct DPAllocations {
    colors: Box<[Color]>,
    path_data: Box<[Vec2]>,
}

pub fn draw_prediction(
    allocations: &mut DPAllocations,
    prediction: &VecDeque<Vec2>,
    num_objects: usize,
    num_steps: usize,
    color: Color,
    fade: bool,
) {
    if prediction.is_empty() {
        return;
    }

    allocations.colors.fill(color);

    if fade {
        for step in 1..num_steps {
            allocations.colors[step].a = 1.0 - (step as f32 / num_steps as f32);
        }
    }

    for obj_idx in 0..num_objects {
        for step in 0..num_steps {
            allocations.path_data[step] = prediction[step * num_objects + obj_idx];
        }

        draw_path(&allocations.path_data, 3.0 / 1000.0, &allocations.colors);
    }
}

// Based on macroquad source
fn draw_path(points: &[Vec2], thickness: f32, colours: &[Color]) {
    if points.len() < 2 {
        return;
    }

    // GL is intrinsically unsafe
    let ctx = unsafe { get_internal_gl().quad_gl };

    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for (idx, window) in points.windows(2).enumerate() {
        let p1 = &window[0];
        let p2 = &window[1];

        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        // Find normal
        let nx = -dy;
        let ny = dx;

        // Normalize to half thickness
        let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
        if tlen < f32::EPSILON {
            continue; // Degenerate little boy
        }
        let tx = nx / tlen;
        let ty = ny / tlen;

        let base = vertices.len() as u32;

        vertices.push(Vertex::new(p1.x + tx, p1.y + ty, 0., 0., 0., colours[idx]));
        vertices.push(Vertex::new(p1.x - tx, p1.y - ty, 0., 0., 0., colours[idx]));
        vertices.push(Vertex::new(
            p2.x + tx,
            p2.y + ty,
            0.,
            0.,
            0.,
            colours[idx + 1],
        ));
        vertices.push(Vertex::new(
            p2.x - tx,
            p2.y - ty,
            0.,
            0.,
            0.,
            colours[idx + 1],
        ));

        indices.extend_from_slice(&[
            base.try_into().unwrap(),
            (base + 1).try_into().unwrap(),
            (base + 2).try_into().unwrap(),
            (base + 2).try_into().unwrap(),
            (base + 1).try_into().unwrap(),
            (base + 3).try_into().unwrap(),
        ]);
    }

    if vertices.is_empty() {
        return;
    }

    ctx.texture(None);
    ctx.draw_mode(DrawMode::Triangles);
    ctx.geometry(&vertices, &indices);
}

pub fn draw(state: &mut State) {
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("Simulation Control").default_pos(egui::Pos2::new(400.0, 200.0))
                .show(egui_ctx, |ui| {
                    ui.label(
                    "Right click to delete. While paused objects can be moved. After pressing create click anywhere to create a new mass.");

                    ui.label("Status:");
                    if ui.button(state.mode.status()).clicked() {
                        match state.mode {
                            Mode::Simulating => state.mode = Mode::Paused,
                            Mode::Paused => state.mode = Mode::Simulating,
                        }
                    }

                    if ui.button("Create new mass").clicked() {
                        state.mouse_state = MouseStatus::CreatingStart;
                    }

                    ui.add(egui::Slider::new(&mut state.time_multiplier, -1f32..=4f32).text("Time warp (-1x - 4x)"));

                    ui.separator();

                    // FW Time prediction
                    if ui.checkbox(&mut state.predict_future, "Predict future").changed() {
                        state.prediction_dirty = true;
                    }
                    ui.add_enabled_ui(state.predict_future, |fwui| {
                        let fw_epoch_label = fwui.label("Max Δepoch");
                        if fwui.add(egui::DragValue::new(&mut state.fw_predict_d_epoch)).labelled_by(fw_epoch_label.id).changed() {
                            state.prediction_dirty = true;
                        }
                        fwui.add(egui::Checkbox::new(&mut state.fw_orbit_line_fade, "Fade FW line"));
                    });

                    // BW Time prediction
                    if ui.checkbox(&mut state.predict_past, "Predict past").changed() {
                        state.prediction_dirty = true;
                    }
                    ui.add_enabled_ui(state.predict_past, |bwui| {
                        let bw_epoch_label = bwui.label("Max Δepoch");
                        if bwui.add(egui::DragValue::new(&mut state.bw_predict_d_epoch)).labelled_by(bw_epoch_label.id).changed() {
                            state.prediction_dirty = true;
                        }
                        bwui.add(egui::Checkbox::new(&mut state.bw_orbit_line_fade, "Fade BW line"));
                    });

                    ui.label(format!("Current physics step: {}", state.fixed_dt));
                    ui.label(format!("Physics compute debt: {}", state.physics_accumulator));
                });

        let mut remove_object = false;
        let mut removed_object_index = 0;
        if let Some(ctx_now) = state.ctx_menu.as_mut() {
            let ctx_menu_window = egui::Window::new(format!("Mass {}", ctx_now.object))
                .fixed_pos(ctx_now.position)
                .collapsible(false)
                .show(egui_ctx, |ui| {
                    if ui.button("Delete").clicked() {
                        removed_object_index = ctx_now.object;
                        remove_object = true;
                        state.prediction_dirty = true;
                    }
                    let mass_label = ui.label("Object mass / kg");
                    if ui
                        .add(egui::DragValue::new(
                            &mut state.objects.mass[ctx_now.object],
                        ))
                        .labelled_by(mass_label.id)
                        .changed()
                    {
                        state.prediction_dirty = true;
                    }

                    ui.label("Position (s) / m");
                    ui.columns(2, |colui| {
                        if colui[0]
                            .add(egui::DragValue::new(
                                &mut state.objects.position_x[ctx_now.object],
                            ))
                            .changed()
                        {
                            state.prediction_dirty = true;
                        }
                        if colui[1]
                            .add(egui::DragValue::new(
                                &mut state.objects.position_y[ctx_now.object],
                            ))
                            .changed()
                        {
                            state.prediction_dirty = true;
                        }
                    });

                    ui.label("Velocity (v) / ms^-1");
                    ui.columns(2, |colui| {
                        if colui[0]
                            .add(egui::DragValue::new(
                                &mut state.objects.velocity_x[ctx_now.object],
                            ))
                            .changed()
                        {
                            state.prediction_dirty = true;
                        };
                        if colui[1]
                            .add(egui::DragValue::new(
                                &mut state.objects.velocity_y[ctx_now.object],
                            ))
                            .changed()
                        {
                            state.prediction_dirty = true;
                        };
                    });
                });

            ctx_now.interaction_rect = ctx_menu_window.unwrap().response.interact_rect;
        }
        if remove_object {
            state.objects.remove_object(removed_object_index);
            state.ctx_menu = None;
        }
    });

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
                for i in 0..state.objects.len() {
                    let radius = Circle::new(
                        state.objects.position_x[i],
                        state.objects.position_y[i],
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
                state
                    .objects
                    .insert_object(position, Vec2::new(0.0, 0.0), 1000000.0);

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
                state.objects.position_x[index] = position.x;
                state.objects.position_y[index] = position.y;
                state.prediction_dirty = true;
            }
        }
    }

    // Open ctx menu
    // TODO: Performance Gainz by evaluating circle around the mouse
    if is_mouse_button_down(MouseButton::Right) {
        let mut found_index = None;
        for i in 0..state.objects.len() {
            let radius = Circle::new(
                state.objects.position_x[i],
                state.objects.position_y[i],
                5.0 / 1000.0,
            );
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

    if state.prediction_dirty {
        if state.predict_future {
            let max_steps = (state.fw_predict_d_epoch / state.fixed_dt).round() as usize;
            state.future_predictor = Some(Predictor {
                objects: state.objects.clone(),
                objects_terminal: state.objects.clone(),
                path: VecDeque::with_capacity(max_steps * state.objects.len()),
                steps_completed: 0,
                max_steps,
                direction: PredictionDirection::Future,
            });

            state.future_predictor.as_mut().unwrap().simulate_steps(
                &mut state.y4_integrator,
                state.fixed_dt,
                state.objects.len(),
            );

            state.fw_pred_d_allocs = Some(DPAllocations {
                // The color given here is actually completely irrelevent
                colors: vec![GREEN; state.objects.len() * max_steps].into_boxed_slice(),
                path_data: vec![Vec2::ZERO; max_steps].into_boxed_slice(),
            });
        } else {
            state.future_predictor = None;
            state.fw_pred_d_allocs = None;
        }
        if state.predict_past {
            let max_steps = (state.bw_predict_d_epoch / state.fixed_dt).round() as usize;
            state.past_predictor = Some(Predictor {
                objects: state.objects.clone(),
                objects_terminal: state.objects.clone(),
                path: VecDeque::with_capacity(max_steps * state.objects.len()),
                steps_completed: 0,
                max_steps,
                direction: PredictionDirection::Past,
            });

            state.past_predictor.as_mut().unwrap().simulate_steps(
                &mut state.y4_integrator,
                -state.fixed_dt,
                state.objects.len(),
            ); // This simulation leaves the terminal, ready to be rolled
            //
            state.bw_pred_d_allocs = Some(DPAllocations {
                colors: vec![RED; state.objects.len() * max_steps].into_boxed_slice(),
                path_data: vec![Vec2::ZERO; max_steps].into_boxed_slice(),
            });
        } else {
            state.past_predictor = None;
            state.bw_pred_d_allocs = None;
        }
        state.prediction_dirty = false;
    }

    if let Some(pred) = &state.future_predictor {
        let allocs = state.fw_pred_d_allocs.as_mut().unwrap();
        draw_prediction(
            allocs,
            &pred.path,
            pred.objects.len(),
            pred.max_steps,
            GREEN,
            state.fw_orbit_line_fade,
        );
    }
    if let Some(pred) = &state.past_predictor {
        let allocs = state.bw_pred_d_allocs.as_mut().unwrap();
        draw_prediction(
            allocs,
            &pred.path,
            pred.objects.len(),
            pred.max_steps,
            RED,
            state.bw_orbit_line_fade,
        );
    }
}
