use macroquad::{color::BLACK, shapes::draw_circle};

use crate::objects::Objects;

pub fn draw_objects(objects: &Objects) {
    for i in 0..objects.len() {
        draw_circle(
            objects.position_x[i],
            objects.position_y[i],
            5.0 / 1000.0,
            BLACK,
        );
    }
}
