use crate::{
    MouseStatus,
    state::{Mode, State},
};

pub fn simulation_panel(ui_ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Simulation Control").default_pos(egui::Pos2::new(400.0, 200.0))
                .show(ui_ctx, |ui| {
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
}
