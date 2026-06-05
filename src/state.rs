use std::sync::{Arc, RwLock, atomic::AtomicBool};

use crate::{
    r#async::OrbitAnalysisService,
    objects::Objects,
    phys::{Predictor, Y4Integrator, orbit_analysis::OrbitAnalysisResult},
    render::DPAllocations,
    ui::{camera::CameraController, context_menu::CTXMenu},
};

pub enum Mode {
    Simulating,
    Paused,
}

impl Mode {
    pub fn status(&self) -> String {
        match &self {
            Mode::Simulating => "SIMULATING".to_string(),
            Mode::Paused => "PAUSED".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MouseStatus {
    Released,
    Dragging(usize), // I have blood on my hands
    Held,
    Creating,
    CreatingStart,
}

pub struct State {
    pub objects: Arc<RwLock<Objects>>,
    pub ut: f32,

    pub mode: Mode,
    pub time_multiplier: f32,

    pub predict_future: bool,
    pub fw_predict_d_epoch: f32,
    pub fw_orbit_line_fade: bool,

    pub future_predictor: Option<Predictor>,
    pub fw_pred_d_allocs: Option<DPAllocations>,

    pub predict_past: bool,
    pub bw_predict_d_epoch: f32,
    pub bw_orbit_line_fade: bool,

    pub past_predictor: Option<Predictor>,
    pub bw_pred_d_allocs: Option<DPAllocations>,

    pub prediction_dirty: bool,

    pub orbit_analysis_result: Arc<RwLock<Option<OrbitAnalysisResult>>>,
    pub orbit_analysis_service: OrbitAnalysisService,
    pub draw_kepler_conic: Arc<AtomicBool>,
    pub analysis_secondary: Arc<RwLock<usize>>,
    pub analysis_enabled: bool,
    pub analysis_window_open: bool,

    pub mouse_state: MouseStatus,

    pub ctx_menu: Option<CTXMenu>,

    pub camera_controller: CameraController,

    pub y4_integrator: Y4Integrator,
    pub physics_accumulator: f32,
    pub fixed_dt: f32,
}
