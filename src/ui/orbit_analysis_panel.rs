use egui::{Checkbox, ProgressBar, RichText};

use crate::state::State;

fn format_f32_option(input: Option<f32>) -> String {
    input
        .map(|val| val.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn data_row(ui: &mut egui::Ui, string_1: &'static str, string_2: String) {
    ui.label(string_1);
    ui.label(string_2);
    ui.end_row();
}

pub fn draw_orbit_analysis_panel(ui_ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Orbit Analysis").vscroll(true).show(ui_ctx, |ui| {
        // Use monospace as a fallback font since it has superscript minus (U+207B)
        let mut fonts = egui::FontDefinitions::default();
        if let Some(monospace_fonts) = fonts.families.get(&egui::FontFamily::Monospace) {
            let mono_fallback_names = monospace_fonts.clone();

            if let Some(proportional_fonts) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                for font_name in mono_fallback_names {
                    if !proportional_fonts.contains(&font_name) {
                        proportional_fonts.push(font_name);
                    }
                }
            }
        }
        ui_ctx.set_fonts(fonts);

        ui.checkbox(&mut state.analysis_enabled, "Run orbital analysis");
        {
            let status = state.orbit_analysis_service.status.read().unwrap();
            ui.add(ProgressBar::new(status.state.f32_complete()).text(status.state.to_string()));
        }

        // This is a giant grid
        let ui_builder = egui::UiBuilder::new();
        ui.scope_builder(ui_builder, |ui| {
            egui::Grid::new("orbit_grid").num_columns(2).spacing([20.0, 2.0]).striped(true).show(ui, |ui| {
                if state.analysis_window_open {
                    let orbit_analysis = state.orbit_analysis_result.read().unwrap();
                    if let Some(orbit_analysis) = &*orbit_analysis {
                        ui.label(RichText::new("Body information:").italics().size(20.0));
                        ui.separator();
                        ui.end_row();
                        data_row(ui, "Selected Secondary", format!("{}", orbit_analysis.secondary));
                        data_row(ui, "Computed Primary", format!("{}", orbit_analysis.primary));
                        data_row(ui, "Orbit Modality", orbit_analysis.orbit_type.to_string().into());

                        ui.end_row();
                        ui.label(RichText::new("Basic information:").italics().size(20.0));
                        ui.separator();
                        ui.end_row();
                        data_row(ui, "Altitude", format!("{} m", orbit_analysis.altitude));
                        data_row(ui, "Orbital Speed", format!("{} ms⁻¹", orbit_analysis.orbital_speed));
                        data_row(ui, "Computed Escape Velocity", format!("{} ms⁻¹", orbit_analysis.escape_velocity));
                        data_row(ui, "Computed Circular Velocity", format!("{} ms⁻¹", orbit_analysis.circular_velocity));
                        data_row(ui, "Speed relative to escape", format!("{}x", orbit_analysis.speed_relative_to_escape));
                        data_row(ui, "Speed relative to circular", format!("{}x", orbit_analysis.speed_relative_to_circular));

                        ui.end_row();
                        ui.label(RichText::new("Keplerian Orbit Information:").italics().size(20.0));
                        ui.separator();
                        ui.end_row();
                        data_row(ui, "Apoapsis", format!("{} m", format_f32_option(orbit_analysis.apoapsis)));
                        data_row(ui, "Time to Apoapsis", format!("{} s", format_f32_option(orbit_analysis.ap_time)));
                        data_row(ui, "Periapsis", format!("{} m", orbit_analysis.periapsis));
                        data_row(ui, "Time to Periapsis", format!("{} s", format_f32_option(orbit_analysis.pe_time)));
                        data_row(ui, "Argument of Periapsis (ω)", format!("{}", orbit_analysis.argument_of_periapsis));
                        data_row(ui, "Osculating period", format!("{} s", orbit_analysis.period));
                        data_row(ui, "Semi-Major Axis", format!("{} m", orbit_analysis.SMA));
                        data_row(ui, "Semi-minor Axis", format!("{} m", orbit_analysis.SmA));
                        data_row(ui, "Eccentricity [Magnitude]", format!("{}", orbit_analysis.eccentricity));
                        data_row(ui, "Eccentricity [Vector]", format!("{}", orbit_analysis.eccentricity_vector));
                        data_row(ui, "True Anomaly", format!("{}", orbit_analysis.true_anomaly));
                        data_row(ui, "Mean Motion", format!("{}", format_f32_option(orbit_analysis.mean_motion)));

                        // TODO: Add ability to draw conic based on keplerian analysis
                        ui.add_enabled(false, Checkbox::new(&mut false, "Draw Kepler conic [INOP]"));
                        ui.end_row();

                        ui.label(RichText::new("Miscellaneous:").italics().size(20.0));
                        ui.separator();
                        ui.end_row();
                        data_row(ui, "Specific Energy", format!("{} J", orbit_analysis.specific_energy));
                        data_row(ui, "Specific Angular Momentum", format!("{} Js", orbit_analysis.specific_angular_momentum));
                        data_row(ui, "Radial Velocity", format!("{} ms⁻¹", orbit_analysis.radial_velocity));

                        ui.separator();
                        ui.separator();
                        ui.end_row();

                        ui.collapsing("Analysis Information", |ui| {
                            let status = state.orbit_analysis_service.status.read().unwrap();
                            ui.label(format!("Last runtime {}ms", status.last_runtime_ms));
                            ui.label(format!("Last update {:?}", status.last_update));
                            ui.label(format!("Analyses Completed {}", status.analyses_completed));

                            ui.label("Candidate score is the measure the orbit analyser uses to dertermine the orbital primary. The body with the lowest score is picked (or if two bodies have the same score the one with the lower index) is picked for analysis. Score is based on several factors. For bound orbits it's the osculating period divided by 100. If the orbit is unbound the distance to the secondary is used as the score");
                            ui.label("Possible Primary Scores:");
                                ui.horizontal(|ui| {
                                    ui.label("Body Index:");
                                    ui.label("Body Score:");
                                });
                                ui.end_row();
                            for primary_candidate_score in &orbit_analysis.primary_candidate_scores {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", primary_candidate_score.0));
                                    ui.label(format!("{}", primary_candidate_score.1));
                                });
                                ui.end_row();
                            }
                        });
                    } else {
                        ui.label(RichText::new("No analysis/Computing...").italics().size(20.0));
                    }
                }
            })
        });
    });
}
