use macroquad::prelude::*;

use crate::{
    phys::Y4Integrator,
    ui::{CTXMenu, CameraController},
};

mod phys;
mod ui;

struct State {
    objects: Objects,
    ut: f32,

    mode: Mode,
    time_multiplier: f32,

    predict_future: bool,
    fw_predict_pts: f32,
    fw_predict_d_epoch: f32,
    fw_orbit_line_fade: bool,

    future_prediction: Option<Vec<Vec2>>,

    predict_past: bool,
    bw_predict_pts: f32,
    bw_predict_d_epoch: f32,
    bw_orbit_line_fade: bool,

    past_prediction: Option<Vec<Vec2>>,

    prediction_dirty: bool,

    mouse_state: MouseStatus,

    ctx_menu: Option<CTXMenu>,

    camera_controller: CameraController,

    y4_integrator: Y4Integrator,
}

#[derive(Debug, Default, Clone)]
struct Objects {
    position_x: Vec<f32>,
    position_y: Vec<f32>,

    velocity_x: Vec<f32>,
    velocity_y: Vec<f32>,

    mass: Vec<f32>,
}

impl Objects {
    pub fn len(&self) -> usize {
        self.mass.len()
    }

    pub fn insert_object(&mut self, position: Vec2, velocity: Vec2, mass: f32) {
        self.position_x.push(position.x);
        self.position_y.push(position.y);

        self.velocity_x.push(velocity.x);
        self.velocity_y.push(velocity.y);

        self.mass.push(mass);
    }

    pub fn remove_object(&mut self, idx: usize) {
        self.position_x.remove(idx);
        self.position_y.remove(idx);

        self.velocity_x.remove(idx);
        self.velocity_y.remove(idx);

        self.mass.remove(idx);
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
        objects: Objects::default(),
        ut: 0.0,

        mode: Mode::Paused,
        time_multiplier: 1.0,

        predict_future: true,
        fw_predict_pts: 1000.0,
        fw_predict_d_epoch: 20.0,
        fw_orbit_line_fade: false,

        future_prediction: None,

        predict_past: false,
        bw_predict_pts: 1000.0,
        bw_predict_d_epoch: 20.0,
        bw_orbit_line_fade: true,

        past_prediction: None,

        prediction_dirty: true,

        mouse_state: MouseStatus::Released,

        ctx_menu: None,

        camera_controller: CameraController::new(),

        y4_integrator: Y4Integrator::default(),
    };

    // https://astronomy.stackexchange.com/questions/50297/initial-state-for-a-3-body-problem-to-create-figure-8-restricted-to-2d
    // Since G scales so quickly the masses either have to be enormous or the distances scaled

    // Three body problem solution (Requires ~~rk4~~ for sufficient quality)
    // Create a mass
    state.objects.insert_object(
        Vec2::new(0.9700436, -0.24308753),
        Vec2::new(0.4662037, 0.43236573),
        1.498e10,
    );

    // Create another mass
    state.objects.insert_object(
        -Vec2::new(state.objects.position_x[0], state.objects.position_y[0]),
        Vec2::new(state.objects.velocity_x[0], state.objects.velocity_y[0]),
        state.objects.mass[0],
    );

    // Guess what
    state.objects.insert_object(
        Vec2::new(0.0, 0.0),
        -2.0 * Vec2::new(state.objects.velocity_x[0], state.objects.velocity_y[0]),
        state.objects.mass[0],
    );

    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        state.camera_controller.update();
        set_camera(&state.camera_controller.camera); // Worldspace rendering

        ui::draw(&mut state);

        match state.mode {
            Mode::Simulating => {
                state
                    .y4_integrator
                    .step(&mut state.objects, get_frame_time() * state.time_multiplier);
                state.ut += get_frame_time() * state.time_multiplier
            }
            Mode::Paused => {}
        }

        ui::draw_objects(&state.objects);

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
