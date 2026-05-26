use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use crate::{
    utils::{pack_state, unpack_state},
    Object,
};

fn compute_acceleration(objects: &[Object], acc: &mut Vec<Vec2>) {
    const G: f32 = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32;
    const SOFTENING_TERM: f32 = 1e-6;
    let n = objects.len();

    acc.clear();
    acc.resize(objects.len(), Vec2::ZERO);

    for i in 0..n {
        for j in (i + 1)..n {
            let r = objects[j].position - objects[i].position;

            let dist_sq = r.length_squared() + SOFTENING_TERM;
            let inv_dist = dist_sq.sqrt().recip();
            let inv_dist3 = inv_dist * inv_dist * inv_dist;
            let force_dir = r * inv_dist3;

            acc[i] += force_dir * (G * objects[j].mass);
            acc[j] -= force_dir * (G * objects[i].mass);
        }
    }
}

fn derivatives_from_objects(acc: &mut Vec<Vec2>, objects: &[Object], deriv: &mut Vec<f32>) {
    let n = objects.len();
    deriv.resize(n * 4, 0.0);
    compute_acceleration(objects, acc);
    for i in 0..n {
        let base = i * 4;
        // derivative of position = velocity
        deriv[base] = objects[i].velocity.x;
        deriv[base + 1] = objects[i].velocity.y;
        // derivative of velocity = acceleration
        deriv[base + 2] = acc[i].x;
        deriv[base + 3] = acc[i].y;
    }
}

pub struct RK4Integrator {
    buffs: RK4Buffers,
}

struct RK4Buffers {
    acceleration: Vec<Vec2>,
    state: Vec<f32>,
    temp: Vec<f32>,
    k1: Vec<f32>,
    k2: Vec<f32>,
    k3: Vec<f32>,
    k4: Vec<f32>,
    temp_objects: Vec<Object>,
}

impl RK4Integrator {
    pub fn new() -> RK4Integrator {
        RK4Integrator {
            buffs: RK4Buffers {
                acceleration: vec![],
                state: vec![],
                temp: vec![],
                k1: vec![],
                k2: vec![],
                k3: vec![],
                k4: vec![],
                temp_objects: vec![],
            },
        }
    }
    // Yes blud, I am solving 4th order
    pub fn step(&mut self, objects: &mut [Object], ut: &mut f32, dt: f32, time_multiplier: f32) {
        let actual_dt = dt * time_multiplier;
        let n = objects.len();
        pack_state(objects, &mut self.buffs.state);
        let state_buff_len = self.buffs.state.len();
        self.buffs.temp.resize(state_buff_len, 0.0);

        // k1
        derivatives_from_objects(&mut self.buffs.acceleration, objects, &mut self.buffs.k1);

        // k2
        for i in 0..state_buff_len {
            self.buffs.temp[i] = self.buffs.state[i] + 0.5 * actual_dt * self.buffs.k1[i];
        }
        self.buffs.temp_objects.resize(n, Object::default());
        Self::populate_temp_objects(n, &self.buffs.temp, objects, &mut self.buffs.temp_objects);
        derivatives_from_objects(
            &mut self.buffs.acceleration,
            &self.buffs.temp_objects,
            &mut self.buffs.k2,
        );

        // k3
        for i in 0..state_buff_len {
            self.buffs.temp[i] = self.buffs.state[i] + 0.5 * actual_dt * self.buffs.k2[i];
        }
        Self::populate_temp_objects(n, &self.buffs.temp, objects, &mut self.buffs.temp_objects);
        derivatives_from_objects(
            &mut self.buffs.acceleration,
            &self.buffs.temp_objects,
            &mut self.buffs.k3,
        );

        // k4
        for i in 0..state_buff_len {
            self.buffs.temp[i] = self.buffs.state[i] + actual_dt * self.buffs.k3[i];
        }
        Self::populate_temp_objects(n, &self.buffs.temp, objects, &mut self.buffs.temp_objects);
        derivatives_from_objects(
            &mut self.buffs.acceleration,
            &self.buffs.temp_objects,
            &mut self.buffs.k4,
        );

        // Final update
        for i in 0..state_buff_len {
            self.buffs.state[i] += actual_dt
                * (self.buffs.k1[i]
                    + 2.0 * self.buffs.k2[i]
                    + 2.0 * self.buffs.k3[i]
                    + self.buffs.k4[i])
                / 6.0;
        }
        unpack_state(&self.buffs.state, objects);

        *ut += actual_dt;
    }

    fn populate_temp_objects(n: usize, temp: &[f32], source: &[Object], out: &mut [Object]) {
        (0..n).for_each(|i| {
            let base = i * 4;

            out[i].position = Vec2::new(temp[base], temp[base + 1]);
            out[i].velocity = Vec2::new(temp[base + 2], temp[base + 3]);
            out[i].mass = source[i].mass;
        });
    }
}

pub fn predict(
    rk4_integrator: &mut RK4Integrator,
    initial_conditions: &[Object],
    predict_pts: i32,
    predict_d_epoch: f32,
) -> Vec<Vec2> {
    let body_count = initial_conditions.len();
    let mut prediction: Vec<Vec2> = vec![Vec2::ZERO; (predict_pts as usize) * body_count];
    let time_step = predict_d_epoch / predict_pts as f32; // TODO: Adaptively size steps (not to be taken lightly)
    let mut running_conditions = initial_conditions.to_owned();
    let mut ut: f32 = 0.0;

    for step in 0..predict_pts {
        rk4_integrator.step(&mut running_conditions, &mut ut, time_step, 1.0);
        for (body_idx, _body) in running_conditions.iter().enumerate() {
            prediction[step as usize * body_count + body_idx] =
                running_conditions[body_idx].position;
        }
    }

    prediction
}
