use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use crate::{objects::Objects, phys::orbit_analysis::OrbitAnalysisResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisState {
    Sleeping,
    Calculating,
    Finished,
}

impl AnalysisState {
    pub fn to_string(&self) -> String {
        match self {
            AnalysisState::Sleeping => "Sleeping".to_owned(),
            AnalysisState::Calculating => "Calculating".to_owned(),
            AnalysisState::Finished => "Finished".to_owned(),
        }
    }
    pub fn f32_complete(&self) -> f32 {
        match self {
            AnalysisState::Sleeping => 1.0,
            AnalysisState::Calculating => 0.5,
            AnalysisState::Finished => 1.0,
        }
    }
}

#[derive(Debug)]
pub struct AnalysisMeta {
    pub state: AnalysisState,
    pub last_runtime_ms: f32,
    pub last_update: Instant,
    pub analyses_completed: u64,
}

impl Default for AnalysisMeta {
    fn default() -> Self {
        Self {
            state: AnalysisState::Finished,
            last_runtime_ms: 0.0,
            last_update: Instant::now(),
            analyses_completed: 0,
        }
    }
}

#[derive(Debug)]
pub struct OrbitAnalysisService {
    thread: Option<std::thread::JoinHandle<()>>,
    pub running: Arc<AtomicBool>,

    pub status: Arc<RwLock<AnalysisMeta>>,
    latest_result: Arc<RwLock<Option<OrbitAnalysisResult>>>,

    objects: Arc<RwLock<Objects>>,
    secondary: Arc<RwLock<usize>>,
    draw_kepler_conic: Arc<AtomicBool>,
}

impl OrbitAnalysisService {
    pub fn new(
        objects: Arc<RwLock<Objects>>,
        analysis_result: Arc<RwLock<Option<OrbitAnalysisResult>>>,
        secondary: Arc<RwLock<usize>>,
        draw_kepler_conic: Arc<AtomicBool>,
    ) -> Self {
        Self {
            thread: None,
            running: Arc::new(AtomicBool::new(false)),
            status: Arc::new(RwLock::new(AnalysisMeta::default())),
            latest_result: analysis_result,
            objects,
            secondary,
            draw_kepler_conic,
        }
    }

    pub fn start(&mut self) {
        if self.thread.is_some() {
            panic!("Tried to start analysis thread while it was already running");
        }

        self.running.store(true, Ordering::Relaxed);

        let running = self.running.clone();
        let status = self.status.clone();
        let latest_result = self.latest_result.clone();
        let objects = self.objects.clone();
        let secondary = self.secondary.clone();
        let draw_kepler_conic = self.draw_kepler_conic.clone();

        self.thread = Some(std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                {
                    status.write().unwrap().state = AnalysisState::Calculating;
                }

                let start = Instant::now();

                let objects_clone = objects.read().unwrap().clone();
                let secondary_clone = secondary.read().unwrap().clone();
                let draw_kepler_conic = draw_kepler_conic.load(Ordering::Relaxed);

                let result = {
                    OrbitAnalysisResult::analyse_orbits(
                        &objects_clone,
                        secondary_clone,
                        draw_kepler_conic,
                    )
                };

                {
                    let mut metadata = status.write().unwrap();
                    metadata.last_runtime_ms = (start.elapsed().as_secs_f64() * 1000.0) as f32;
                    metadata.analyses_completed += 1;
                    metadata.last_update = Instant::now();
                    metadata.state = AnalysisState::Sleeping;
                }

                {
                    *latest_result.write().unwrap() = result;
                }

                // TODO:: Make this true deltatime
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }));
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for OrbitAnalysisService {
    fn drop(&mut self) {
        self.stop();
    }
}
