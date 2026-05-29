#[cfg(target_arch = "wasm32")]
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

#[cfg(target_arch = "wasm32")]
use crate::{Objects, phys::acceleration::Acceleration};

#[cfg(target_arch = "wasm32")]
pub fn compute_acceleration(objects: &Objects, acc: &mut Acceleration) {
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
