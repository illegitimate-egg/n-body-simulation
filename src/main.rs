#![cfg_attr(not(target_arch = "wasm32"), feature(portable_simd))]
#![windows_subsystem = "windows"]

use macroquad::{conf::Conf, prelude::*};

use crate::{
    phys::{Predictor, Y4Integrator},
    ui::{CTXMenu, CameraController, DPAllocations},
};

mod phys;
mod ui;

struct State {
    objects: Objects,
    ut: f32,

    mode: Mode,
    time_multiplier: f32,

    predict_future: bool,
    fw_predict_d_epoch: f32,
    fw_orbit_line_fade: bool,

    future_predictor: Option<Predictor>,
    fw_pred_d_allocs: Option<DPAllocations>,

    predict_past: bool,
    bw_predict_d_epoch: f32,
    bw_orbit_line_fade: bool,

    past_predictor: Option<Predictor>,
    bw_pred_d_allocs: Option<DPAllocations>,

    prediction_dirty: bool,

    mouse_state: MouseStatus,

    ctx_menu: Option<CTXMenu>,

    camera_controller: CameraController,

    y4_integrator: Y4Integrator,
    physics_accumulator: f32,
    fixed_dt: f32,
}

#[derive(Debug, Default, Clone)]
struct Objects {
    position_x: Box<[f32]>,
    position_y: Box<[f32]>,

    velocity_x: Box<[f32]>,
    velocity_y: Box<[f32]>,

    mass: Box<[f32]>,
}

impl Objects {
    pub fn new(number_of_initial_bodies: usize) -> Objects {
        Objects {
            position_x: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            position_y: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            velocity_x: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            velocity_y: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            mass: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.mass.len()
    }

    pub fn insert_object(&mut self, position: Vec2, velocity: Vec2, mass: f32) {
        let mut expanded_position_x = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_position_y = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_velocity_x = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_velocity_y = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_mass = vec![0.0; self.len() + 1].into_boxed_slice();

        for i in 0..self.len() {
            expanded_position_x[i] = self.position_x[i];
            expanded_position_y[i] = self.position_y[i];
            expanded_velocity_x[i] = self.velocity_x[i];
            expanded_velocity_y[i] = self.velocity_y[i];
            expanded_mass[i] = self.mass[i];
        }

        expanded_position_x[self.len()] = position.x;
        expanded_position_y[self.len()] = position.y;
        expanded_velocity_x[self.len()] = velocity.x;
        expanded_velocity_y[self.len()] = velocity.y;
        expanded_mass[self.len()] = mass;

        self.position_x = expanded_position_x;
        self.position_y = expanded_position_y;
        self.velocity_x = expanded_velocity_x;
        self.velocity_y = expanded_velocity_y;
        self.mass = expanded_mass;
    }

    // Just realloc and shift stuff around so the target doesn't exist anymore
    pub fn remove_object(&mut self, idx: usize) {
        let mut shrunk_position_x = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_position_y = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_velocity_x = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_velocity_y = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_mass = vec![0.0; self.len() - 1].into_boxed_slice();

        for i in 0..idx {
            shrunk_position_x[i] = self.position_x[i];
            shrunk_position_y[i] = self.position_y[i];
            shrunk_velocity_x[i] = self.velocity_x[i];
            shrunk_velocity_y[i] = self.velocity_y[i];
            shrunk_mass[i] = self.mass[i];
        }

        for i in idx + 1..self.len() {
            shrunk_position_x[i - 1] = self.position_x[i];
            shrunk_position_y[i - 1] = self.position_y[i];
            shrunk_velocity_x[i - 1] = self.velocity_x[i];
            shrunk_velocity_y[i - 1] = self.velocity_y[i];
            shrunk_mass[i - 1] = self.mass[i];
        }

        self.position_x = shrunk_position_x;
        self.position_y = shrunk_position_y;
        self.velocity_x = shrunk_velocity_x;
        self.velocity_y = shrunk_velocity_y;
        self.mass = shrunk_mass;
    }

    pub fn total_momentum(&self) -> Vec2 {
        let mut momentum = Vec2::ZERO;

        for i in 0..self.len() {
            momentum.x += self.mass[i] * self.velocity_x[i];
            momentum.y += self.mass[i] * self.velocity_y[i];
        }

        momentum
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        let mut kinetic_energy = 0.0;

        for i in 0..self.len() {
            let velocity_squared =
                self.velocity_x[i] * self.velocity_x[i] + self.velocity_y[i] + self.velocity_y[i];

            kinetic_energy += 0.5 * self.mass[i] * velocity_squared;
        }

        kinetic_energy
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

fn window_conf() -> Conf {
    Conf {
        miniquad_conf: miniquad::conf::Conf::default(),
        update_on: None,
        default_filter_mode: FilterMode::Linear,
        draw_call_vertex_capacity: 70_000,
        draw_call_index_capacity: 70_000,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // LaTeX maths font available at https://svn.tug.org:8369/texlive/trunk/Master/texmf-dist/fonts/opentype/public/lm/lmmath.otf?revision=23153&pathrev=23153&view=markup
    let maths_font =
        load_ttf_font_from_bytes(include_bytes!("../fonts/lmmath-regular.otf.ttf")).unwrap();
    set_default_font(maths_font);

    let mut state = State {
        objects: Objects::new(3), // 3 body problem
        ut: 0.0,

        mode: Mode::Paused,
        time_multiplier: 1.0,

        predict_future: true,
        fw_predict_d_epoch: 20.0,
        fw_orbit_line_fade: false,
        future_predictor: None,
        fw_pred_d_allocs: None,

        predict_past: false,
        bw_predict_d_epoch: 20.0,
        bw_orbit_line_fade: true,
        past_predictor: None,
        bw_pred_d_allocs: None,

        prediction_dirty: true,

        mouse_state: MouseStatus::Released,

        ctx_menu: None,

        camera_controller: CameraController::new(),

        y4_integrator: Y4Integrator::new(3),
        physics_accumulator: 0.0,
        fixed_dt: 240.0f32.recip(),
    };

    // https://astronomy.stackexchange.com/questions/50297/initial-state-for-a-3-body-problem-to-create-figure-8-restricted-to-2d
    // Since G scales so quickly the masses either have to be enormous or the distances scaled

    // Three body problem solution (Requires ~~rk4~~ for sufficient quality)
    // The first 3 entries are already allocated and ready to be written to
    // Create a mass
    state.objects.position_x[0] = 0.9700436;
    state.objects.position_y[0] = -0.24308753;
    state.objects.velocity_x[0] = 0.4662037;
    state.objects.velocity_y[0] = 0.43236573;
    state.objects.mass[0] = 1.498e10;

    // Create another mass
    state.objects.position_x[1] = -state.objects.position_x[0];
    state.objects.position_y[1] = -state.objects.position_y[0];
    state.objects.velocity_x[1] = state.objects.velocity_x[0];
    state.objects.velocity_y[1] = state.objects.velocity_y[0];
    state.objects.mass[1] = state.objects.mass[0];

    // Guess what
    state.objects.position_x[2] = 0.0;
    state.objects.position_y[2] = 0.0;
    state.objects.velocity_x[2] = -2.0 * state.objects.velocity_x[0];
    state.objects.velocity_y[2] = -2.0 * state.objects.velocity_y[0];
    state.objects.mass[2] = state.objects.mass[0];

    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        state.camera_controller.update();
        set_camera(&state.camera_controller.camera); // Worldspace rendering

        ui::draw(&mut state);

        match state.mode {
            Mode::Simulating => {
                let frame_dt = get_frame_time();

                state.physics_accumulator += frame_dt * state.time_multiplier.abs();

                let signed_dt = state.fixed_dt * state.time_multiplier.signum();

                // If we're really far behind on physics, give on going realtime and just lag for a while
                state.physics_accumulator = state.physics_accumulator.min(0.25);

                while state.physics_accumulator >= state.fixed_dt {
                    state.y4_integrator.step(&mut state.objects, signed_dt);

                    if let Some(pred) = state.future_predictor.as_mut() {
                        pred.advance(&mut state.y4_integrator, signed_dt, state.objects.len());
                    }
                    if let Some(pred) = state.past_predictor.as_mut() {
                        pred.advance(&mut state.y4_integrator, signed_dt, state.objects.len());
                    }

                    state.ut += signed_dt;

                    state.physics_accumulator -= state.fixed_dt;
                }
            }

            Mode::Paused => {}
        }

        ui::draw_objects(&state.objects);

        set_default_camera(); // Switch to screenspace rendering

        draw_text(
            format! {"dt: {}", get_frame_time()},
            20.0,
            20.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            format! {"fps: {}", get_frame_time().recip()},
            20.0,
            80.0,
            30.0,
            DARKGRAY,
        );
        draw_text(format! {"ut: {}", state.ut}, 20.0, 50.0, 30.0, DARKGRAY);
        draw_text(
            format! {"{}x zoom", state.camera_controller.camera.zoom},
            20.0,
            500.0,
            20.0,
            DARKGRAY,
        );

        let momentum = state.objects.total_momentum();
        draw_text(
            format! {"Σp = ({:.5e}, {:.5e})kgms^-1", momentum.x, momentum.y},
            20.0,
            200.0,
            20.0,
            DARKGRAY,
        );
        draw_text(
            format! {"|Σp| = {:.5e}kgms^-1", momentum.length()},
            20.0,
            220.0,
            20.0,
            DARKGRAY,
        );

        draw_text(
            format! {"ΣE_k = {:.5e}J", state.objects.total_kinetic_energy()},
            20.0,
            240.0,
            20.0,
            DARKGRAY,
        );

        egui_macroquad::draw();
        next_frame().await;
    }
}
