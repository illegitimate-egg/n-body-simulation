use macroquad::prelude::*;

use crate::{phys::rk4_step, ui::{CTXMenu, CameraController}};

mod phys;
mod ui;
mod utils;

struct State {
    objects: Vec<Object>,
    ut: f32,

    mode: Mode,
    time_multiplier: f32,

    predict_future: bool,
    fw_predict_pts: f32,
    fw_predict_d_epoch: f32,
    fw_orbit_line_fade: bool,

    predict_past: bool,
    bw_predict_pts: f32,
    bw_predict_d_epoch: f32,
    bw_orbit_line_fade: bool,

    mouse_state: MouseStatus,

    ctx_menu: Option<CTXMenu>,

    camera_controller: CameraController,
}

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
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            mass: 1.0e10, // 1 billion blistering kilograms (1 Megaton)
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

#[macroquad::main("n-body")]
async fn main() {
    let mut state = State {
        objects: vec![],
        ut: 0.0,

        mode: Mode::Paused,
        time_multiplier: 1.0,

        predict_future: true,
        fw_predict_pts: 1000.0,
        fw_predict_d_epoch: 20.0,
        fw_orbit_line_fade: false,

        predict_past: false,
        bw_predict_pts: 1000.0,
        bw_predict_d_epoch: 20.0,
        bw_orbit_line_fade: true,

        mouse_state: MouseStatus::Released,

        ctx_menu: None,

        camera_controller: CameraController::new()
    };
    
    // https://astronomy.stackexchange.com/questions/50297/initial-state-for-a-3-body-problem-to-create-figure-8-restricted-to-2d
    // Since G scales so quickly the masses either have to be enormous or the distances scaled

    // Three body problem solution (Requires rk4 for sufficient quality)
    // Create a mass
    state.objects.push(Object {
        position: Vec2::new(0.9700436, -0.24308753),
        velocity: Vec2::new(0.4662037, 0.43236573),
        mass: 1.498e10,
    });

    // Create another mass
    state.objects.push(Object {
        position: -state.objects[0].position,
        velocity: state.objects[0].velocity,
        mass: state.objects[0].mass,
    });

    // Guess what
    state.objects.push(Object {
        position: Vec2::new(0.0, 0.0),
        velocity: -2.0 * state.objects[0].velocity,
        mass: state.objects[0].mass,
    });

    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        state.camera_controller.update();
        set_camera(&state.camera_controller.camera); // Worldspace rendering

        ui::draw(&mut state);

        match state.mode {
            Mode::Simulating => rk4_step(
                &mut state.objects,
                &mut state.ut,
                get_frame_time(),
                state.time_multiplier,
            ),
            Mode::Paused => {}
        }

        ui::draw_objects(&mut state.objects);

        set_default_camera(); // Switch to screenspace rendering

        draw_text(
            &format! {"dt: {}", get_frame_time()}.to_string(),
            20.0,
            20.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            &format! {"ut: {}", state.ut}.to_string(),
            20.0,
            50.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            &format! {"{}x zoom", state.camera_controller.camera.zoom}.to_string(),
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
