use egui::Checkbox;

use crate::{phys::orbit_analysis, state::State};

fn format_f32_option(input: Option<f32>) -> String {
    input
        .map(|val| val.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

pub fn draw_orbit_analysis_panel(ui_ctx: &egui::Context, state: &mut State) {
    egui::Window::new("Orbit Analysis").show(ui_ctx, |ui| {
        // We should use a monospace font
        // 14.0pts is the default egui font size
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Monospace),
        );

        if state.analysis_enabled {
            state.orbit_analysis_result = orbit_analysis::OrbitAnalysisResult::analyse_orbits(
                &state.objects,
                state.analysis_secondary,
            );

            if let Some(orbit_analysis) = &state.orbit_analysis_result {
                ui.label("Body information:");
                ui.label(format!("Selected Secondary: {}", orbit_analysis.secondary));
                ui.label(format!("Computed Primary:   {}", orbit_analysis.primary));
                ui.label(format!("Orbit Modality:     {}", orbit_analysis.orbit_type.to_string()));

                ui.separator();
                ui.label("Basic information:");
                ui.label(format!("Altitude (m):                       {}", orbit_analysis.altitude));
                ui.label(format!("Orbital Speed (ms^-1):              {}", orbit_analysis.orbital_speed));
                ui.label(format!("Computed Escape Velocity (ms^-1):   {}", orbit_analysis.escape_velocity));
                ui.label(format!("Computed Circular Velocity (ms^-1): {}", orbit_analysis.circular_velocity));
                ui.label(format!("Speed relative to escape:           {}x", orbit_analysis.speed_relative_to_escape));
                ui.label(format!("Speed relative to circular:         {}x", orbit_analysis.speed_relative_to_circular));

                ui.separator();
                ui.label("Keplerian Orbit Information:");
                ui.label(format!("Apoapsis (m):              {}", format_f32_option(orbit_analysis.apoapsis)));
                ui.label(format!("Time to Apoapsis (s):      {}", format_f32_option(orbit_analysis.ap_time)));
                ui.label(format!("Periapsis (m):             {}", orbit_analysis.periapsis));
                ui.label(format!("Time to Periapsis (s):     {}", format_f32_option(orbit_analysis.pe_time)));
                ui.label(format!("Argument of Periapsis (ω): {}", orbit_analysis.argument_of_periapsis));
                ui.label(format!("Osculating period (s):     {}", orbit_analysis.period));
                ui.label(format!("Semi-Major Axis (m):       {}", orbit_analysis.SMA));
                ui.label(format!("Semi-minor Axis (m):       {}", orbit_analysis.SmA));
                ui.label(format!("Eccentricity [Magnitude]:  {}", orbit_analysis.eccentricity));
                ui.label(format!("Eccentricity [Vector]:     {}", orbit_analysis.eccentricity_vector));
                ui.label(format!("True Anomaly:              {}", orbit_analysis.true_anomaly));
                ui.label(format!("Mean Motion:               {}", format_f32_option(orbit_analysis.mean_motion)));

                // TODO: Add ability to draw conic based on keplerian analysis
                ui.add_enabled(false, Checkbox::new(&mut false, "Draw Kepler conic [INOP]"));

                ui.separator();
                ui.label("Miscellaneous:");
                ui.label(format!("Specific Energy (J):            {}", orbit_analysis.specific_energy));
                ui.label(format!("Specific Angular Momentum (Js): {}", orbit_analysis.specific_angular_momentum));
                ui.label(format!("Radial Velocity (ms^-1):        {}", orbit_analysis.radial_velocity));

                ui.separator();
                ui.collapsing("Analysis Information", |ui| {
                    ui.label("Candidate score is the measure the orbit analyser uses to dertermine the orbital primary. The body with the lowest score is picked (or if two bodies have the same score the one with the lower index) is picked for analysis. Score is based on several factors. For bound orbits it's the osculating period divided by 100. If the orbit is unbound the distance to the secondary is used as the score");
                    ui.label("Possible Primary Scores:");
                        ui.horizontal(|ui| {
                            ui.label("Body Index:");
                            ui.label("Body Score:");
                        });
                    for primary_candidate_score in &orbit_analysis.primary_candidate_scores {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", primary_candidate_score.0));
                            ui.label(format!("{}", primary_candidate_score.1));
                        });
                    }
                });
            } else {
                println!("No analysis/Computing...");
            }
        }
    });
}
