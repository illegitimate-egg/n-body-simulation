use crate::{
    state::State,
    ui::{
        context_menu::draw_ctx_menu, interaction::handle_interaction, panels::simulation_panel,
        prediction::draw_prediction_ui,
    },
};

// Based on macroquad source
pub fn draw(state: &mut State) {
    egui_macroquad::ui(|egui_ctx| {
        simulation_panel(&egui_ctx, state);
        draw_ctx_menu(&egui_ctx, state);
    });

    handle_interaction(state);

    draw_prediction_ui(state);
}
