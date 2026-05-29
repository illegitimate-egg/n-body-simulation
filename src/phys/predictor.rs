use std::collections::VecDeque;

use macroquad::math::Vec2;

use crate::{objects::Objects, phys::Y4Integrator};

pub enum PredictionDirection {
    Future, // El futuro
    Past,
}

pub struct Predictor {
    pub objects: Objects,          // Prediction head, furthest point in future
    pub objects_terminal: Objects, // Prediction tail, right before the jaws of hell
    pub path: VecDeque<Vec2>,
    pub steps_completed: usize,
    pub max_steps: usize,
    pub direction: PredictionDirection,
}

impl Predictor {
    fn signed_dt(&self, dt: f32) -> f32 {
        match self.direction {
            PredictionDirection::Future => dt.abs(),
            PredictionDirection::Past => -dt.abs(),
        }
    }

    pub fn simulate_steps(&mut self, integrator: &mut Y4Integrator, dt: f32, body_count: usize) {
        let signed_dt = self.signed_dt(dt);

        for _ in 0..self.max_steps {
            integrator.step(&mut self.objects, signed_dt);

            for body_idx in 0..body_count {
                self.path.push_back(Vec2::new(
                    self.objects.position_x[body_idx],
                    self.objects.position_y[body_idx],
                ));
            }

            self.steps_completed += 1;
        }
    }

    pub fn advance(&mut self, integrator: &mut Y4Integrator, dt: f32, body_count: usize) {
        match self.direction {
            PredictionDirection::Future => {
                self.advance_fw(integrator, dt, body_count);
            }
            PredictionDirection::Past => {
                self.advance_bw(integrator, dt, body_count);
            }
        }
    }

    // FW logic:
    // Structure's first index stores most recent
    // Structure's last index stores most latent
    // +dt:
    //  - Remove first step
    //  - Shift all steps backwards 1 step
    //  - Step objects and terminal
    //  - Insert at end
    // -dt:
    //  - Remove last step
    //  - Shift all remaining steps forwards
    //  - Step backwards from objects_terminal
    //  - Insert at beginning
    fn advance_fw(&mut self, integrator: &mut Y4Integrator, dt: f32, body_count: usize) {
        // If dt is positive then we're going forwards
        if dt.signum() > 0.0 {
            self.path.drain(..body_count);

            integrator.step(&mut self.objects, dt);
            integrator.step(&mut self.objects_terminal, dt);

            for body_idx in 0..body_count {
                self.path.push_back(Vec2::new(
                    self.objects.position_x[body_idx],
                    self.objects.position_y[body_idx],
                ));
            }
        } else {
            // Bendy was a little devil thing
            for _ in 0..body_count {
                self.path.pop_back();
            }

            integrator.step(&mut self.objects, dt);
            integrator.step(&mut self.objects_terminal, dt);

            for body_idx in (0..body_count).rev() {
                self.path.push_front(Vec2::new(
                    self.objects_terminal.position_x[body_idx],
                    self.objects_terminal.position_y[body_idx],
                ));
            }
        }
    }

    // BW logic:
    // Structure's first index stores most recent
    // Structure's last index stores least recent
    // +dt:
    //  - Remove last step
    //  - Shift all steps backwards 1 step
    //  - Step objects
    //  - Insert objects_terminal at front
    // -dt:
    //  - Remove first step
    //  - Shift all remaining steps backwards
    //  - Step backwards from objects
    //  - Insert at back
    fn advance_bw(&mut self, integrator: &mut Y4Integrator, dt: f32, body_count: usize) {
        // If dt is positive then we're going forwards
        if dt.signum() > 0.0 {
            for _ in 0..body_count {
                self.path.pop_back();
            }

            integrator.step(&mut self.objects, dt);
            integrator.step(&mut self.objects_terminal, dt);

            for body_idx in (0..body_count).rev() {
                self.path.push_front(Vec2::new(
                    self.objects_terminal.position_x[body_idx],
                    self.objects_terminal.position_y[body_idx],
                ));
            }
        } else {
            self.path.drain(..body_count);

            integrator.step(&mut self.objects, dt);
            integrator.step(&mut self.objects_terminal, dt);

            for body_idx in 0..body_count {
                self.path.push_back(Vec2::new(
                    self.objects.position_x[body_idx],
                    self.objects.position_y[body_idx],
                ));
            }
        }
    }
}
