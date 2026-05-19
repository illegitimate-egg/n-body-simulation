use std::{any::Any, rc::Rc};

use macroquad::prelude::*;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

// All masses are singularities
#[derive(Debug, Clone, Copy)]
struct Object {
    position: Vec2, // ms^-1
    velocity: Vec2, // ms^-1
    mass: f32,      // kg
}

impl Default for Object {
    fn default() -> Self {
        Self {
            position: Vec2::new(2.0, 2.0),
            velocity: Vec2::new(0.0, 0.0),
            mass: 1.0, // 1 blistering kilogram
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CTXMenu {
    object: usize,
    position: egui::Pos2,
    interaction_rect: egui::Rect,
}

struct CameraController {
    camera: Camera2D,
    dragging: bool,
    last_mouse: Vec2,
}

impl CameraController {
    fn new() -> Self {
        Self {
            camera: Camera2D {
                zoom: vec2(2.0 / screen_width() * 1000.0, -2.0 / screen_height() * 1000.0),
                target: vec2(2.0, 2.0),
                ..Default::default()
            },
            dragging: false,
            last_mouse: mouse_position().into(),
        }
    }

    fn update(&mut self) {
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
            self.camera.zoom.y = self.camera.zoom.y.clamp(-10.0, -0.0005);

            // World position AFTER zoom
            let world_after = self.camera.screen_to_world(mouse_screen);

            // Move camera target so cursor stays fixed on same world point
            self.camera.target += world_before - world_after;
        }
    }
}

enum Mode {
    Simulating,
    Paused,
}

#[derive(Debug, PartialEq)]
enum MouseStatus {
    Released,
    Dragging(usize), // I have blood on my hands
    Held,
    Creating,
    CreatingStart,
}

impl Mode {
    fn status(&self) -> String {
        match &self {
            Mode::Simulating => "SIMULATING".to_string(),
            Mode::Paused => "PAUSED".to_string(),
        }
    }
}

fn physics_step(object_vector: &mut [Object], ut: &mut f32, dt: f32, time_multiplier: f32) {
    let vector_state = object_vector.to_owned();

    for object in object_vector.iter_mut().enumerate() {
        let index = object.0;
        let object = object.1;

        // Apply forces (and their accelerations)
        // |F| = m_1 |a|
        // |F| = G(m_1 m_2) / r^2
        // r = sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)
        // We want this in terms of a
        // |a| = G(m_2) / r^2
        // |a| = G(m_2) / (m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2
        // Since the vector a will point towards m_2
        // Get unit vector pointing towards the second mass
        // butt (vec[2]) = [(m_1.x - m_2.x)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2), (m_1.y - m_2.y)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)];
        // Multiply it by the vector a to get the acceleration vector
        // a = |a| * butt
        // v = v += a * dt

        let mut a = Vec2::new(0.0, 0.0);

        for object_two in vector_state.iter() {
            if vector_state[index].position == object_two.position {
                continue;
            }

            let r_squared = (object.position.x as f64 - object_two.position.x as f64).powi(2)
                + (object.position.y as f64 - object_two.position.y as f64).powi(2);

            let mag_a: f64 =
                (NEWTONIAN_CONSTANT_OF_GRAVITATION * object_two.mass as f64) / r_squared;
            a += Vec2::new(
                -mag_a as f32 * (object.position.x - object_two.position.x)
                    / r_squared.sqrt() as f32,
                -mag_a as f32 * (object.position.y - object_two.position.y)
                    / r_squared.sqrt() as f32,
            );
        }

        // TODO: Update integrator

        // Apply the acceleration to the velocity
        object.velocity += a * dt * time_multiplier;
        // Make velocity velocit
        object.position += object.velocity * dt * time_multiplier;
    }

    *ut += dt * time_multiplier;
}

fn draw_state(object_vector: &mut Vec<Object>) {
    for object in object_vector {
        // Draw it
        draw_circle(
            object.position.x,
            object.position.y,
            5.0/1000.0,
            BLACK,
        ); // Draw a thing
    }
}

// Outer vector is each step, inner vector is each object. Object indexs are constant
fn draw_prediction(
    prediction: Vec<Vec<Object>>,
    color: Color,
    fade: bool,
) {
    let number_of_bodies = prediction[0].len(); // Should be a constant size
    let number_of_timesteps = prediction.len();

    // Important this is an index and not an object
    for object in 0..number_of_bodies {
        let mut x_0 = prediction[0][object].position.x;
        let mut y_0 = prediction[0][object].position.y;

        // First step is explicit case since there is insufficient data
        for point in 1..number_of_timesteps {
            let x_1 = prediction[point][object].position.x;
            let y_1 = prediction[point][object].position.y;

            if fade {
                let mut fade_color = color;
                fade_color.a = 1.0 - (point as f32 / number_of_timesteps as f32);
                draw_line(
                    x_0,
                    y_0,
                    x_1,
                    y_1,
                    1.5/1000.0,
                    fade_color,
                );
            } else {
                draw_line(
                    x_0,
                    y_0,
                    x_1,
                    y_1,
                    1.5/1000.0,
                    color,
                );
            }

            x_0 = x_1;
            y_0 = y_1;
        }
    }
}

fn predict(
    initial_conditions: &[Object],
    predict_pts: i32,
    predict_d_epoch: f32,
) -> Vec<Vec<Object>> {
    let mut prediction: Vec<Vec<Object>> = vec![];

    let time_step = predict_d_epoch / predict_pts as f32;

    let mut running_conditions = initial_conditions.to_owned();
    let mut ut: f32 = 0.0;

    for _point in 0..predict_pts {
        physics_step(&mut running_conditions, &mut ut, time_step, 1.0);

        prediction.push(running_conditions.clone());
    }

    prediction
}

#[macroquad::main("n-body")]
async fn main() {
    let mut object_vector: Vec<Object> = vec![];
    let mut ut: f32 = 0.0;

    // Create a mass
    object_vector.push(Object {
        velocity: Vec2::new(0.0, 0.02),
        ..Default::default()
    });

    // Create another mass
    object_vector.push(Object {
        position: Vec2::new(2.1, 2.0),
        velocity: Vec2::new(0.0, 0.0),
        mass: 1000000.0,
    });

    let mut mode = Mode::Paused;
    let mut time_multiplier = 1.0;

    let mut predict_future = true;
    let mut fw_predict_pts: f32 = 1000.0; // This being f32 makes me sad
    let mut fw_predict_d_epoch: f32 = 20.0;
    let mut fw_orbit_line_fade = false;

    let mut predict_past = false;
    let mut bw_predict_pts: f32 = 1000.0; // This being f32 makes me sad
    let mut bw_predict_d_epoch: f32 = 20.0;
    let mut bw_orbit_line_fade = true;

    let mut mouse_state = MouseStatus::Released;

    let mut ctx_menu: Option<CTXMenu> = None;

    let mut camera_controller = CameraController::new();

    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        camera_controller.update();
        set_camera(&camera_controller.camera);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Simulation Control").default_pos(egui::Pos2::new(400.0, 200.0))
                .show(egui_ctx, |ui| {
                    ui.label(
                    "Right click to delete. While paused objects can be moved. After pressing create click anywhere to create a new mass.");

                    ui.label("Status:");
                    if ui.button(mode.status()).clicked() {
                        match mode {
                            Mode::Simulating => mode = Mode::Paused,
                            Mode::Paused => mode = Mode::Simulating,
                        }
                    }

                    if ui.button("Create new mass").clicked() {
                        mouse_state = MouseStatus::CreatingStart;
                    }

                    ui.add(egui::Slider::new(&mut time_multiplier, -1f32..=4f32).text("Time warp (-1x - 4x)"));

                    ui.separator();

                    // FW Time prediction
                    ui.checkbox(&mut predict_future, "Predict future");
                    ui.add_enabled_ui(predict_future, |fwui| {
                        let fw_pts_label = fwui.label("FW Simulation points");
                        fwui.add(egui::Slider::new(&mut fw_predict_pts, 10f32..=10000f32).step_by(1.0)).labelled_by(fw_pts_label.id);
                        let fw_epoch_label = fwui.label("Max Δepoch");
                        fwui.add(egui::DragValue::new(&mut fw_predict_d_epoch)).labelled_by(fw_epoch_label.id);
                        fwui.add(egui::Checkbox::new(&mut fw_orbit_line_fade, "Fade FW line"));
                    });

                    // BW Time prediction
                    ui.checkbox(&mut predict_past, "Predict past");
                    ui.add_enabled_ui(predict_past, |bwui| {
                        let bw_pts_label = bwui.label("BW Simulation points");
                        bwui.add(egui::Slider::new(&mut bw_predict_pts, 10f32..=10000f32).step_by(1.0)).labelled_by(bw_pts_label.id);
                        let bw_epoch_label = bwui.label("Max Δepoch");
                        bwui.add(egui::DragValue::new(&mut bw_predict_d_epoch)).labelled_by(bw_epoch_label.id);
                        bwui.add(egui::Checkbox::new(&mut bw_orbit_line_fade, "Fade BW line"));
                    });
                });

            if let Some(ctx_now) = ctx_menu {
                let ctx_menu_window = egui::Window::new(format!("Mass {}", ctx_now.object))
                    .fixed_pos(ctx_now.position)
                    .collapsible(false)
                    .show(egui_ctx, |ui| {
                        if ui.button("Delete").clicked() {
                            object_vector.remove(ctx_now.object);
                            ctx_menu = None;
                        }
                        let mass_label = ui.label("Object mass / kg");
                        ui.add(egui::DragValue::new(
                            &mut object_vector[ctx_now.object].mass,
                        ))
                        .labelled_by(mass_label.id);

                        ui.label("Position (s) / m");
                        ui.columns(2, |colui| {
                            colui[0].add(egui::DragValue::new(
                                &mut object_vector[ctx_now.object].position.x,
                            ));
                            colui[1].add(egui::DragValue::new(
                                &mut object_vector[ctx_now.object].position.y,
                            ));
                        });

                        ui.label("Velocity (v) / ms^-1");
                        ui.columns(2, |colui| {
                            colui[0].add(egui::DragValue::new(
                                &mut object_vector[ctx_now.object].velocity.x,
                            ));
                            colui[1].add(egui::DragValue::new(
                                &mut object_vector[ctx_now.object].velocity.y,
                            ));
                        });
                    });

                ctx_menu = Some(CTXMenu {
                    object: ctx_now.object,
                    position: ctx_now.position,
                    interaction_rect: ctx_menu_window.unwrap().response.interact_rect,
                });
            }
        });

        if is_mouse_button_down(MouseButton::Left) {
            match mouse_state {
                MouseStatus::Released => {
                    if let Some(ctx) = ctx_menu
                        && !(ctx.interaction_rect.min.x < mouse_position().0
                            && ctx.interaction_rect.max.x + ctx.interaction_rect.min.x
                                > mouse_position().0
                            && ctx.interaction_rect.min.y < mouse_position().1
                            && ctx.interaction_rect.max.y + ctx.interaction_rect.min.y
                                > mouse_position().1)
                    {
                        ctx_menu = None;
                    }

                    let mut found_index = None;
                    for (i, object) in object_vector.iter_mut().enumerate() {
                        let radius = Circle::new(
                            object.position.x,
                            object.position.y,
                            5.0 / 1000.0, // 1000 is the zoom factor
                        );
                        if radius.contains(&camera_controller.camera.screen_to_world(mouse_position().into())) {
                            found_index = Some(i);
                            break;
                        }
                    }

                    mouse_state = match found_index {
                        Some(idx) => MouseStatus::Dragging(idx),
                        None => MouseStatus::Held,
                    };
                }
                MouseStatus::Creating => {
                    object_vector.push(Object {
                        position: camera_controller.camera.screen_to_world(mouse_position().into()),
                        velocity: Vec2::new(0.0, 0.0),
                        mass: 1000000.0,
                    });

                    mouse_state = MouseStatus::Released;
                }
                _ => {}
            }
        } else {
            if mouse_state == MouseStatus::CreatingStart {
                mouse_state = MouseStatus::Creating;
            } else if mouse_state == MouseStatus::Creating {
                let world_mouse = camera_controller.camera.screen_to_world(mouse_position().into());
                draw_circle(
                    world_mouse.x,
                    world_mouse.y,
                    5.0/1000.0,
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    },
                );
            } else {
                mouse_state = MouseStatus::Released;
            }
        }

        match mode {
            Mode::Simulating => {}
            Mode::Paused => {
                if let MouseStatus::Dragging(index) = mouse_state {
                    object_vector[index].position = camera_controller.camera.screen_to_world(mouse_position().into());
                }
            }
        }

        // Open ctx menu
        if is_mouse_button_down(MouseButton::Right) {
            let mut found_index = None;
            for (i, object) in object_vector.iter_mut().enumerate() {
                let radius = Circle::new(
                    object.position.x,
                    object.position.y,
                    5.0 / 1000.0,
                );
                if radius.contains(&camera_controller.camera.screen_to_world(mouse_position().into())) {
                    found_index = Some(i);
                    break;
                } else {
                    ctx_menu = None;
                }
            }

            if let Some(idx) = found_index {
                ctx_menu = Some(CTXMenu {
                    object: idx,
                    position: egui::Pos2::new(mouse_position().0, mouse_position().1),
                    interaction_rect: egui::Rect {
                        min: egui::Pos2::new(0.0, 0.0),
                        max: egui::Pos2::new(0.0, 0.0),
                    },
                });
            }
        }

        if predict_future {
            draw_prediction(
                predict(
                    &object_vector,
                    fw_predict_pts.round() as i32,
                    fw_predict_d_epoch,
                ),
                GREEN,
                fw_orbit_line_fade,
            );
        }

        if predict_past {
            draw_prediction(
                predict(
                    &object_vector,
                    bw_predict_pts.round() as i32,
                    -bw_predict_d_epoch,
                ),
                RED,
                bw_orbit_line_fade,
            );
        }

        match mode {
            Mode::Simulating => physics_step(
                &mut object_vector,
                &mut ut,
                get_frame_time(),
                time_multiplier,
            ),
            Mode::Paused => {}
        }

        draw_state(&mut object_vector);

        set_default_camera();

        draw_text(
            &format! {"dt: {}", get_frame_time()}.to_string(),
            20.0,
            20.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            &format! {"ut: {}", ut}.to_string(),
            20.0,
            50.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            &format! {"{}x zoom", camera_controller.camera.zoom}.to_string(),
            20.0,
            500.0,
            20.0,
            DARKGRAY,
        );

        egui_macroquad::draw();
        next_frame().await;

        // std::process::exit(1);
    }
}
