use crate::objects::Objects;
use crate::phys::acceleration::Acceleration;

#[cfg(not(target_arch = "wasm32"))]
use crate::phys::acceleration::compute_acceleration;
#[cfg(target_arch = "wasm32")]
use crate::platform::wasm::compute_acceleration;

// const W1: f32 = 1.0 / (2.0 - 2.0_f32.cbrt());
// const W0: f32 = -2.0_f32.cbrt() / (2.0 - 2.0_f32.cbrt());
const W1: f32 = 1.351_207_1;
const W0: f32 = -1.702_414_4;

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
