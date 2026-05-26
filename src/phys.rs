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

fn derivatives_from_objects(acc: &mut Vec<Vec2>, objects: &[Object]) -> Vec<f32> {
    let n = objects.len();
    let mut deriv = vec![0.0; n * 4];
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
    deriv
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
        self.buffs.state = pack_state(objects);
        self.buffs.temp = vec![0.0; self.buffs.state.len()];

        // k1
        self.buffs.k1 = derivatives_from_objects(&mut self.buffs.acceleration, objects);

        // k2
        for i in 0..self.buffs.state.len() {
            self.buffs.temp[i] = self.buffs.state[i] + 0.5 * actual_dt * self.buffs.k1[i];
        }
        self.buffs.temp_objects = (0..n)
            .map(|i| {
                let base = i * 4;
                Object {
                    position: Vec2::new(self.buffs.temp[base], self.buffs.temp[base + 1]),
                    velocity: Vec2::new(self.buffs.temp[base + 2], self.buffs.temp[base + 3]),
                    mass: objects[i].mass, // keep original masses
                }
            })
            .collect();
        self.buffs.k2 =
            derivatives_from_objects(&mut self.buffs.acceleration, &self.buffs.temp_objects);

        // k3
        for i in 0..self.buffs.state.len() {
            self.buffs.temp[i] = self.buffs.state[i] + 0.5 * actual_dt * self.buffs.k2[i];
        }
        self.buffs.temp_objects = (0..n)
            .map(|i| {
                let base = i * 4;
                Object {
                    position: Vec2::new(self.buffs.temp[base], self.buffs.temp[base + 1]),
                    velocity: Vec2::new(self.buffs.temp[base + 2], self.buffs.temp[base + 3]),
                    mass: objects[i].mass,
                }
            })
            .collect();
        self.buffs.k3 =
            derivatives_from_objects(&mut self.buffs.acceleration, &self.buffs.temp_objects);

        // k4
        for i in 0..self.buffs.state.len() {
            self.buffs.temp[i] = self.buffs.state[i] + actual_dt * self.buffs.k3[i];
        }
        self.buffs.temp_objects = (0..n)
            .map(|i| {
                let base = i * 4;
                Object {
                    position: Vec2::new(self.buffs.temp[base], self.buffs.temp[base + 1]),
                    velocity: Vec2::new(self.buffs.temp[base + 2], self.buffs.temp[base + 3]),
                    mass: objects[i].mass,
                }
            })
            .collect();
        self.buffs.k4 =
            derivatives_from_objects(&mut self.buffs.acceleration, &self.buffs.temp_objects);

        // Final update
        for i in 0..self.buffs.state.len() {
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
}

// Marked for execution: old euler cromer integrator
// fn euler_cromer_step(object_vector: &mut [Object], ut: &mut f32, dt: f32, time_multiplier: f32) {
//     let vector_state = object_vector.to_owned();

//     for object in object_vector.iter_mut().enumerate() {
//         let index = object.0;
//         let object = object.1;

//         // Apply forces (and their accelerations)
//         // |F| = m_1 |a|
//         // |F| = G(m_1 m_2) / r^2
//         // r = sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)
//         // We want this in terms of a
//         // |a| = G(m_2) / r^2
//         // |a| = G(m_2) / (m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2
//         // Since the vector a will point towards m_2
//         // Get unit vector pointing towards the second mass
//         // butt (vec[2]) = [(m_1.x - m_2.x)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2), (m_1.y - m_2.y)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)];
//         // Multiply it by the vector a to get the acceleration vector
//         // a = |a| * butt
//         // v = v += a * dt

//         let mut a = Vec2::new(0.0, 0.0);

//         for object_two in vector_state.iter() {
//             if vector_state[index].position == object_two.position {
//                 continue;
//             }

//             let r_squared = (object.position.x as f64 - object_two.position.x as f64).powi(2)
//                 + (object.position.y as f64 - object_two.position.y as f64).powi(2);

//             let r = r_squared.sqrt();

//             let mag_a: f64 =
//                 (NEWTONIAN_CONSTANT_OF_GRAVITATION * object_two.mass as f64) / r_squared;
//             a += Vec2::new(
//                 -mag_a as f32 * (object.position.x - object_two.position.x) / r as f32,
//                 -mag_a as f32 * (object.position.y - object_two.position.y) / r as f32,
//             );
//         }

//         // TODO: Update integrator

//         // Apply the acceleration to the velocity
//         object.velocity += a * dt * time_multiplier;
//         // Make velocity velocit
//         object.position += object.velocity * dt * time_multiplier;
//     }

//     *ut += dt * time_multiplier;
// }

pub fn predict(
    rk4_integrator: &mut RK4Integrator,
    initial_conditions: &[Object],
    predict_pts: i32,
    predict_d_epoch: f32,
) -> Vec<Vec<Vec2>> {
    let mut prediction: Vec<Vec<Vec2>> = Vec::with_capacity(predict_pts as usize);
    let time_step = predict_d_epoch / predict_pts as f32; // TODO: Adaptively size steps (not to be taken lightly)
    let mut running_conditions = initial_conditions.to_owned();
    let mut ut: f32 = 0.0;

    for _ in 0..predict_pts {
        rk4_integrator.step(&mut running_conditions, &mut ut, time_step, 1.0);
        prediction.push(running_conditions.iter().map(|o| o.position).collect());
    }

    prediction
}
