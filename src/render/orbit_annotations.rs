use macroquad::{
    color::{BLUE, WHITE},
    math::Vec2,
    prelude::DrawMode,
    text::{TextParams, draw_text_ex},
    ui::Vertex,
    window::get_internal_gl,
};

pub fn draw_orbit_annotations(position: Vec2, text: &'static str) {
    const DRAW_SCALE: f32 = 20.0f32.recip();

    let ctx = unsafe { get_internal_gl().quad_gl };

    ctx.texture(None);
    ctx.draw_mode(DrawMode::Triangles);
    ctx.geometry(
        &[
            Vertex::new(
                -0.5 * DRAW_SCALE + position.x,
                -0.3 * DRAW_SCALE + position.y,
                0.0,
                0.0,
                0.0,
                BLUE,
            ),
            Vertex::new(0.0 + position.x, 0.0 + position.y, 0.0, 0.0, 0.0, BLUE),
            Vertex::new(
                0.5 * DRAW_SCALE + position.x,
                -0.3 * DRAW_SCALE + position.y,
                0.0,
                0.0,
                0.0,
                BLUE,
            ),
            Vertex::new(
                -0.5 * DRAW_SCALE + position.x,
                -1.0 * DRAW_SCALE + position.y,
                0.0,
                0.0,
                0.0,
                BLUE,
            ),
            Vertex::new(
                0.5 * DRAW_SCALE + position.x,
                -1.0 * DRAW_SCALE + position.y,
                0.0,
                0.0,
                0.0,
                BLUE,
            ),
        ],
        &[0, 1, 2, 0, 3, 2, 3, 2, 4],
    );

    // These values were obtained by brute force because I HATE font parametrics
    draw_text_ex(
        text,
        -0.02 + position.x,
        -0.038 + position.y,
        TextParams {
            font_size: 100,
            font_scale: -0.0003,
            font_scale_aspect: -1.0,
            color: WHITE,
            ..Default::default()
        },
    );
}
