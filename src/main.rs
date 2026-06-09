#![cfg_attr(not(target_arch = "wasm32"), feature(portable_simd))]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};

use macroquad::{conf::Conf, prelude::*};

use crate::{
    r#async::OrbitAnalysisService,
    objects::Objects,
    phys::Y4Integrator,
    render::draw_objects,
    state::{Mode, MouseStatus, State},
    ui::camera::CameraController,
};

// Guh
mod r#async;
mod objects;
mod phys;
mod platform;
mod render;
mod state;
mod ui;

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

    let objects = Arc::new(RwLock::new(Objects::new(3)));

    let orbit_analysis_result = Arc::new(RwLock::new(None));
    let analysis_secondary: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));

    let draw_kepler_conic = Arc::new(AtomicBool::new(false));

    let mut state = State {
        objects: objects.clone(),
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
        orbit_analysis_result: orbit_analysis_result.clone(),
        orbit_analysis_service: OrbitAnalysisService::new(
            objects.clone(),
            orbit_analysis_result.clone(),
            analysis_secondary.clone(),
            draw_kepler_conic.clone(),
        ),
        mouse_state: MouseStatus::Released,
        ctx_menu: None,
        camera_controller: CameraController::new(),
        y4_integrator: Y4Integrator::new(3),
        physics_accumulator: 0.0,
        fixed_dt: 240.0f32.recip(),
        analysis_secondary: analysis_secondary.clone(),
        analysis_enabled: true,
        analysis_window_open: true,
        draw_kepler_conic: draw_kepler_conic.clone(),
    };

    {
        let mut objects = state.objects.write().unwrap();
        // https://astronomy.stackexchange.com/questions/50297/initial-state-for-a-3-body-problem-to-create-figure-8-restricted-to-2d
        // Since G scales so quickly the masses either have to be enormous or the distances scaled

        // Three body problem solution (Requires ~~rk4~~ for sufficient quality)
        // The first 3 entries are already allocated and ready to be written to
        // Create a mass
        objects.position_x[0] = 0.9700436;
        objects.position_y[0] = -0.24308753;
        objects.velocity_x[0] = 0.4662037;
        objects.velocity_y[0] = 0.43236573;
        objects.mass[0] = 1.498e10;

        // Create another mass
        objects.position_x[1] = -objects.position_x[0];
        objects.position_y[1] = -objects.position_y[0];
        objects.velocity_x[1] = objects.velocity_x[0];
        objects.velocity_y[1] = objects.velocity_y[0];
        objects.mass[1] = objects.mass[0];

        // Guess what
        objects.position_x[2] = 0.0;
        objects.position_y[2] = 0.0;
        objects.velocity_x[2] = -2.0 * objects.velocity_x[0];
        objects.velocity_y[2] = -2.0 * objects.velocity_y[0];
        objects.mass[2] = objects.mass[0];
    }

    loop {
        if state.analysis_enabled && state.analysis_window_open {
            if !state.orbit_analysis_service.running.load(Ordering::Relaxed) {
                // Go forth my son, and inherit the Earth
                state.orbit_analysis_service.start();
            }
        } else {
            if state.orbit_analysis_service.running.load(Ordering::Relaxed) {
                state.orbit_analysis_service.stop();
                *state.orbit_analysis_result.write().unwrap() = None;
            }
        }

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

                {
                    let mut objects = state.objects.write().unwrap();
                    while state.physics_accumulator >= state.fixed_dt {
                        state.y4_integrator.step(&mut objects, signed_dt);

                        if let Some(pred) = state.future_predictor.as_mut() {
                            pred.advance(&mut state.y4_integrator, signed_dt, objects.len());
                        }
                        if let Some(pred) = state.past_predictor.as_mut() {
                            pred.advance(&mut state.y4_integrator, signed_dt, objects.len());
                        }

                        state.ut += signed_dt;

                        state.physics_accumulator -= state.fixed_dt;
                    }
                }
            }

            Mode::Paused => {}
        }

        let objects = state.objects.read().unwrap();
        draw_objects(&objects);

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

        let objects = state.objects.read().unwrap();
        let momentum = objects.total_momentum();
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

        let objects = state.objects.read().unwrap();
        draw_text(
            format! {"ΣE_k = {:.5e}J", objects.total_kinetic_energy()},
            20.0,
            240.0,
            20.0,
            DARKGRAY,
        );

        egui_macroquad::draw();
        next_frame().await;
    }
}
