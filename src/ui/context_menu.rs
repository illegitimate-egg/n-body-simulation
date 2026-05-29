use crate::state::State;

#[derive(Debug, Clone, Copy)]
pub struct CTXMenu {
    pub object: usize,
    pub position: egui::Pos2,
    pub interaction_rect: egui::Rect,
}

pub fn draw_ctx_menu(ui_ctx: &egui::Context, state: &mut State) {
    let mut remove_object = false;
    let mut removed_object_index = 0;
    if let Some(ctx_now) = state.ctx_menu.as_mut() {
        let ctx_menu_window = egui::Window::new(format!("Mass {}", ctx_now.object))
            .fixed_pos(ctx_now.position)
            .collapsible(false)
            .show(ui_ctx, |ui| {
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
}
