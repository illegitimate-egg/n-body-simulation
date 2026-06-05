use macroquad::color::BLUE;

use crate::{
    render::{orbit_annotations::draw_orbit_annotations, path_mesh::draw_path},
    state::State,
    ui::{
        context_menu::draw_ctx_menu, interaction::handle_interaction,
        orbit_analysis_panel::draw_orbit_analysis_panel, panels::simulation_panel,
        prediction::draw_prediction_ui,
    },
};

// Based on macroquad source
pub fn draw(state: &mut State) {
    egui_macroquad::ui(|egui_ctx| {
        simulation_panel(egui_ctx, state);
        draw_ctx_menu(egui_ctx, state);

        if state.analysis_window_open {
            draw_orbit_analysis_panel(egui_ctx, state);
        }
    });

    handle_interaction(state);

    draw_prediction_ui(state);

    if let Some(analysis_result) = &*state.orbit_analysis_result.read().unwrap() {
        if let Some(conic) = &analysis_result.conic {
            draw_path(&conic.0, 3.0 / 1000.0, &vec![BLUE; conic.0.len()]);

            draw_orbit_annotations(conic.1.periapsis, "Pe");

            if let Some(ap) = conic.1.apoapsis {
                draw_orbit_annotations(ap, "Ap");
            }
        }
    }
}
