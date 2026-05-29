use std::collections::VecDeque;

use macroquad::{color::Color, math::Vec2};

use crate::render::path_mesh::draw_path;

// Fixed allocations for draw_prediction
pub struct DPAllocations {
    pub colors: Box<[Color]>,
    pub path_data: Box<[Vec2]>,
}

pub fn draw_prediction(
    allocations: &mut DPAllocations,
    prediction: &VecDeque<Vec2>,
    num_objects: usize,
    num_steps: usize,
    color: Color,
    fade: bool,
) {
    if prediction.is_empty() {
        return;
    }

    allocations.colors.fill(color);

    if fade {
        for step in 1..num_steps {
            allocations.colors[step].a = 1.0 - (step as f32 / num_steps as f32);
        }
    }

    for obj_idx in 0..num_objects {
        for step in 0..num_steps {
            allocations.path_data[step] = prediction[step * num_objects + obj_idx];
        }

        draw_path(&allocations.path_data, 3.0 / 1000.0, &allocations.colors);
    }
}
