use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use crate::{
    utils::{pack_state, unpack_state},
    Object,
};

fn compute_acceleration(objects: &[Object]) -> Vec<Vec2> {
    let n = objects.len();
    let mut acc = vec![Vec2::ZERO; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let r = objects[j].position - objects[i].position;
            let r2 = r.length_squared();
            let r3 = r2 * r.length();
            acc[i] += r * (NEWTONIAN_CONSTANT_OF_GRAVITATION as f32 * objects[j].mass / r3);
        }
    }
    acc
}

fn derivatives_from_objects(objects: &[Object]) -> Vec<f32> {
    let n = objects.len();
    let mut deriv = vec![0.0; n * 4];
    let acc = compute_acceleration(objects);
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

// Yes blud, I am solving 4th order
pub fn rk4_step(objects: &mut [Object], ut: &mut f32, dt: f32, time_multiplier: f32) {
    let actual_dt = dt * time_multiplier;
    let n = objects.len();
    let mut state = pack_state(objects);
    let mut temp_state = vec![0.0; state.len()];

    // k1
    let k1 = derivatives_from_objects(objects);

    // k2
    for i in 0..state.len() {
        temp_state[i] = state[i] + 0.5 * actual_dt * k1[i];
    }
    let objects_k2: Vec<Object> = (0..n)
        .map(|i| {
            let base = i * 4;
            Object {
                position: Vec2::new(temp_state[base], temp_state[base + 1]),
                velocity: Vec2::new(temp_state[base + 2], temp_state[base + 3]),
                mass: objects[i].mass, // keep original masses
            }
        })
        .collect();
    let k2 = derivatives_from_objects(&objects_k2);

    // k3
    for i in 0..state.len() {
        temp_state[i] = state[i] + 0.5 * actual_dt * k2[i];
    }
    let objects_k3: Vec<Object> = (0..n)
        .map(|i| {
            let base = i * 4;
            Object {
                position: Vec2::new(temp_state[base], temp_state[base + 1]),
                velocity: Vec2::new(temp_state[base + 2], temp_state[base + 3]),
                mass: objects[i].mass,
            }
        })
        .collect();
    let k3 = derivatives_from_objects(&objects_k3);

    // k4
    for i in 0..state.len() {
        temp_state[i] = state[i] + actual_dt * k3[i];
    }
    let objects_k4: Vec<Object> = (0..n)
        .map(|i| {
            let base = i * 4;
            Object {
                position: Vec2::new(temp_state[base], temp_state[base + 1]),
                velocity: Vec2::new(temp_state[base + 2], temp_state[base + 3]),
                mass: objects[i].mass,
            }
        })
        .collect();
    let k4 = derivatives_from_objects(&objects_k4);

    // Final update
    for i in 0..state.len() {
        state[i] += actual_dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;
    }
    unpack_state(&state, objects);

    *ut += actual_dt;
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
    initial_conditions: &[Object],
    predict_pts: i32,
    predict_d_epoch: f32,
) -> Vec<Vec<Object>> {
    let mut prediction: Vec<Vec<Object>> = Vec::with_capacity(predict_pts as usize);
    let time_step = predict_d_epoch / predict_pts as f32;
    let mut running_conditions = initial_conditions.to_owned();
    let mut ut: f32 = 0.0;

    for _ in 0..predict_pts {
        rk4_step(&mut running_conditions, &mut ut, time_step, 1.0);
        prediction.push(running_conditions.clone());
    }

    prediction
}
