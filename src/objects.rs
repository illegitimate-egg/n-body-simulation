use macroquad::math::Vec2;

#[derive(Debug, Default, Clone)]
pub struct Objects {
    pub position_x: Box<[f32]>,
    pub position_y: Box<[f32]>,

    pub velocity_x: Box<[f32]>,
    pub velocity_y: Box<[f32]>,

    pub mass: Box<[f32]>,
}

impl Objects {
    pub fn new(number_of_initial_bodies: usize) -> Objects {
        Objects {
            position_x: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            position_y: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            velocity_x: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            velocity_y: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
            mass: vec![0.0; number_of_initial_bodies].into_boxed_slice(),
        }
    }

    pub fn len(&self) -> usize {
        self.mass.len()
    }

    pub fn insert_object(&mut self, position: Vec2, velocity: Vec2, mass: f32) {
        let mut expanded_position_x = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_position_y = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_velocity_x = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_velocity_y = vec![0.0; self.len() + 1].into_boxed_slice();
        let mut expanded_mass = vec![0.0; self.len() + 1].into_boxed_slice();

        for i in 0..self.len() {
            expanded_position_x[i] = self.position_x[i];
            expanded_position_y[i] = self.position_y[i];
            expanded_velocity_x[i] = self.velocity_x[i];
            expanded_velocity_y[i] = self.velocity_y[i];
            expanded_mass[i] = self.mass[i];
        }

        expanded_position_x[self.len()] = position.x;
        expanded_position_y[self.len()] = position.y;
        expanded_velocity_x[self.len()] = velocity.x;
        expanded_velocity_y[self.len()] = velocity.y;
        expanded_mass[self.len()] = mass;

        self.position_x = expanded_position_x;
        self.position_y = expanded_position_y;
        self.velocity_x = expanded_velocity_x;
        self.velocity_y = expanded_velocity_y;
        self.mass = expanded_mass;
    }

    // Just realloc and shift stuff around so the target doesn't exist anymore
    pub fn remove_object(&mut self, idx: usize) {
        let mut shrunk_position_x = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_position_y = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_velocity_x = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_velocity_y = vec![0.0; self.len() - 1].into_boxed_slice();
        let mut shrunk_mass = vec![0.0; self.len() - 1].into_boxed_slice();

        for i in 0..idx {
            shrunk_position_x[i] = self.position_x[i];
            shrunk_position_y[i] = self.position_y[i];
            shrunk_velocity_x[i] = self.velocity_x[i];
            shrunk_velocity_y[i] = self.velocity_y[i];
            shrunk_mass[i] = self.mass[i];
        }

        for i in idx + 1..self.len() {
            shrunk_position_x[i - 1] = self.position_x[i];
            shrunk_position_y[i - 1] = self.position_y[i];
            shrunk_velocity_x[i - 1] = self.velocity_x[i];
            shrunk_velocity_y[i - 1] = self.velocity_y[i];
            shrunk_mass[i - 1] = self.mass[i];
        }

        self.position_x = shrunk_position_x;
        self.position_y = shrunk_position_y;
        self.velocity_x = shrunk_velocity_x;
        self.velocity_y = shrunk_velocity_y;
        self.mass = shrunk_mass;
    }

    pub fn total_momentum(&self) -> Vec2 {
        let mut momentum = Vec2::ZERO;

        for i in 0..self.len() {
            momentum.x += self.mass[i] * self.velocity_x[i];
            momentum.y += self.mass[i] * self.velocity_y[i];
        }

        momentum
    }

    pub fn total_kinetic_energy(&self) -> f32 {
        let mut kinetic_energy = 0.0;

        for i in 0..self.len() {
            let velocity_squared =
                self.velocity_x[i] * self.velocity_x[i] + self.velocity_y[i] * self.velocity_y[i];

            kinetic_energy += 0.5 * self.mass[i] * velocity_squared;
        }

        kinetic_energy
    }
}
