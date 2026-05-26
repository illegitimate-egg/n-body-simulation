use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

// const W1: f32 = 1.0 / (2.0 - 2.0_f32.cbrt());
// const W0: f32 = -2.0_f32.cbrt() / (2.0 - 2.0_f32.cbrt());
const W1: f32 = 1.351_207_1;
const W0: f32 = -1.702_414_4;

use crate::Objects;

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
    x: Vec<f32>,
    y: Vec<f32>,
}

impl Acceleration {
    fn resize(&mut self, new_len: usize, value: f32) {
        self.x.resize(new_len, value);
        self.y.resize(new_len, value);
    }
    fn fill(&mut self, value: f32) {
        self.x.fill(value);
        self.y.fill(value);
    }
}

#[derive(Default)]
pub struct Y4Integrator {
    acceleration: Acceleration,
}

impl Y4Integrator {
    pub fn step(&mut self, objects: &mut Objects, dt: f32) {
        self.acceleration.resize(objects.len(), 0.0);
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

pub fn predict(
    y4_integrator: &mut Y4Integrator,
    mut running_conditions: Objects,
    predict_pts: i32,
    predict_d_epoch: f32,
) -> Vec<Vec2> {
    let body_count = running_conditions.len();
    let mut prediction: Vec<Vec2> = vec![Vec2::ZERO; (predict_pts as usize) * body_count];
    let time_step = predict_d_epoch / predict_pts as f32; // TODO: Adaptively size steps (not to be taken lightly)

    for step in 0..predict_pts {
        y4_integrator.step(&mut running_conditions, time_step);
        for (body_idx, _body) in running_conditions.mass.iter().enumerate() {
            prediction[step as usize * body_count + body_idx] = Vec2::new(
                running_conditions.position_x[body_idx],
                running_conditions.position_y[body_idx],
            );
        }
    }

    prediction
}
