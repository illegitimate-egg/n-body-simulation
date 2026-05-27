use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::simd::cmp::SimdPartialEq;
#[cfg(not(target_arch = "wasm32"))]
use std::simd::num::SimdFloat;
#[cfg(not(target_arch = "wasm32"))]
use std::simd::{Select, StdFloat, f32x4, i32x4};

// const W1: f32 = 1.0 / (2.0 - 2.0_f32.cbrt());
// const W0: f32 = -2.0_f32.cbrt() / (2.0 - 2.0_f32.cbrt());
const W1: f32 = 1.351_207_1;
const W0: f32 = -1.702_414_4;

use crate::Objects;

#[cfg(not(target_arch = "wasm32"))]
fn compute_acceleration(objects: &Objects, acc: &mut Acceleration) {
    const G: f32 = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32;
    const SOFTENING: f32 = 1e-6;
    let broadcast_g = f32x4::splat(G);
    let n = objects.len();

    acc.fill(0.0);

    for i in 0..n {
        // Broadcast i-body position
        let position_x_i = f32x4::splat(objects.position_x[i]);
        let position_y_i = f32x4::splat(objects.position_y[i]);

        // SIMD accumulators
        let mut acceleration_x = f32x4::splat(0.0);
        let mut acceleration_y = f32x4::splat(0.0);

        let mut j = 0;

        while j + 4 <= n {
            // Load 4 positions
            let position_x_j = f32x4::from([
                objects.position_x[j],
                objects.position_x[j + 1],
                objects.position_x[j + 2],
                objects.position_x[j + 3],
            ]);

            let position_y_j = f32x4::from([
                objects.position_y[j],
                objects.position_y[j + 1],
                objects.position_y[j + 2],
                objects.position_y[j + 3],
            ]);

            let mass_j = f32x4::from([
                objects.mass[j],
                objects.mass[j + 1],
                objects.mass[j + 2],
                objects.mass[j + 3],
            ]);

            let distance_x = position_x_j - position_x_i;
            let distance_y = position_y_j - position_y_i;

            let distance_squared =
                distance_x * distance_x + distance_y * distance_y + f32x4::splat(SOFTENING);

            let inv_distance = distance_squared.sqrt().recip();
            let inv_distance_3 = inv_distance * inv_distance * inv_distance;

            // Masking the SIMD vector so that we can't play with ourselves
            let lane_indices =
                i32x4::from([j as i32, (j + 1) as i32, (j + 2) as i32, (j + 3) as i32]);
            let i_vec = i32x4::splat(i as i32);
            let mask = lane_indices.simd_ne(i_vec);

            let scale = mass_j * broadcast_g * inv_distance_3;
            let masked_scale = mask.select(scale, f32x4::splat(0.0));

            acceleration_x += distance_x * masked_scale;
            acceleration_y += distance_y * masked_scale;

            j += 4;
        }

        acc.x[i] = acceleration_x.reduce_sum();
        acc.y[i] = acceleration_y.reduce_sum();

        while j < n {
            if i != j {
                let dx = objects.position_x[j] - objects.position_x[i];
                let dy = objects.position_y[j] - objects.position_y[i];

                let dist_sq = dx * dx + dy * dy + SOFTENING;

                let inv_dist = dist_sq.sqrt().recip();
                let inv_dist3 = inv_dist * inv_dist * inv_dist;

                let scale = G * objects.mass[j] * inv_dist3;

                acc.x[i] += dx * scale;
                acc.y[i] += dy * scale;
            }

            j += 1;
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn compute_acceleration(objects: &Objects, acc: &mut Acceleration) {
    const G: f32 = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32;
    const SOFTENING_TERM: f32 = 1e-6;
    let n = objects.len();

    acc.fill(0.0);

    for i in 0..n {
        for j in (i + 1)..n {
            let r_x = objects.position_x[j] - objects.position_x[i];
            let r_y = objects.position_y[j] - objects.position_y[i];

            let dist_sq = r_x * r_x + r_y * r_y + SOFTENING_TERM;
            let inv_dist = dist_sq.sqrt().recip();
            let inv_dist3 = inv_dist * inv_dist * inv_dist;
            let force_x = r_x * inv_dist3;
            let force_y = r_y * inv_dist3;

            let j_gravity_coefficient = G * objects.mass[j];
            let i_gravity_coefficient = G * objects.mass[i];
            acc.x[i] += force_x * j_gravity_coefficient;
            acc.y[i] += force_y * j_gravity_coefficient;
            acc.x[j] -= force_x * i_gravity_coefficient;
            acc.y[j] -= force_y * i_gravity_coefficient;
        }
    }
}

fn drift(objects: &mut Objects, dt: f32) {
    for i in 0..objects.len() {
        objects.position_x[i] += objects.velocity_x[i] * dt;
        objects.position_y[i] += objects.velocity_y[i] * dt;
    }
}

fn kick(objects: &mut Objects, acc: &Acceleration, dt: f32) {
    for i in 0..objects.len() {
        objects.velocity_x[i] += acc.x[i] * dt;
        objects.velocity_y[i] += acc.y[i] * dt;
    }
}

#[derive(Default)]
struct Acceleration {
    x: Box<[f32]>,
    y: Box<[f32]>,
}

impl Acceleration {
    fn new(n: usize) -> Self {
        Self {
            x: vec![0.0_f32; n].into_boxed_slice(),
            y: vec![0.0_f32; n].into_boxed_slice(),
        }
    }
    fn fill(&mut self, value: f32) {
        self.x.fill(value);
        self.y.fill(value);
    }
    fn resize(&mut self, new_len: usize, value: f32) {
        self.x = vec![0.0; new_len].into_boxed_slice();
        self.y = vec![0.0; new_len].into_boxed_slice();
        for i in 0..new_len {
            self.x[i] = value;
            self.y[i] = value;
        }
    }
}

#[derive(Default)]
pub struct Y4Integrator {
    acceleration: Acceleration,
}

impl Y4Integrator {
    pub fn new(body_count: usize) -> Self {
        Self {
            acceleration: Acceleration::new(body_count),
        }
    }
}

impl Y4Integrator {
    pub fn step(&mut self, objects: &mut Objects, dt: f32) {
        if self.acceleration.x.len() != objects.len() {
            self.acceleration.resize(objects.len(), 0.0);
        }

        self.leapfrog_step(objects, W1 * dt);
        self.leapfrog_step(objects, W0 * dt);
        self.leapfrog_step(objects, W1 * dt);
    }

    fn leapfrog_step(&mut self, objects: &mut Objects, dt: f32) {
        compute_acceleration(objects, &mut self.acceleration);

        kick(objects, &self.acceleration, dt / 2.0);

        drift(objects, dt);

        compute_acceleration(objects, &mut self.acceleration);

        kick(objects, &self.acceleration, dt / 2.0);
    }
}

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
