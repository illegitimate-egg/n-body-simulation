#[cfg(not(target_arch = "wasm32"))]
use std::simd::cmp::SimdPartialEq;
#[cfg(not(target_arch = "wasm32"))]
use std::simd::num::SimdFloat;
#[cfg(not(target_arch = "wasm32"))]
use std::simd::{Select, StdFloat, f32x4, i32x4};

use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use crate::objects::Objects;

#[derive(Default)]
pub struct Acceleration {
    pub x: Box<[f32]>,
    pub y: Box<[f32]>,
}

impl Acceleration {
    pub fn new(n: usize) -> Self {
        Self {
            x: vec![0.0_f32; n].into_boxed_slice(),
            y: vec![0.0_f32; n].into_boxed_slice(),
        }
    }
    pub fn fill(&mut self, value: f32) {
        self.x.fill(value);
        self.y.fill(value);
    }
    pub fn resize(&mut self, new_len: usize, value: f32) {
        self.x = vec![0.0; new_len].into_boxed_slice();
        self.y = vec![0.0; new_len].into_boxed_slice();
        for i in 0..new_len {
            self.x[i] = value;
            self.y[i] = value;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn compute_acceleration(objects: &Objects, acc: &mut Acceleration) {
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
