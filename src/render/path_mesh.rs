use macroquad::{color::Color, math::Vec2, prelude::DrawMode, ui::Vertex, window::get_internal_gl};

pub fn draw_path(points: &[Vec2], thickness: f32, colours: &[Color]) {
    if points.len() < 2 {
        return;
    }

    // GL is intrinsically unsafe
    let ctx = unsafe { get_internal_gl().quad_gl };

    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for (idx, window) in points.windows(2).enumerate() {
        let p1 = &window[0];
        let p2 = &window[1];

        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        // Find normal
        let nx = -dy;
        let ny = dx;

        // Normalize to half thickness
        let tlen = (nx * nx + ny * ny).sqrt() / (thickness * 0.5);
        if tlen < f32::EPSILON {
            continue; // Degenerate little boy
        }
        let tx = nx / tlen;
        let ty = ny / tlen;

        let base = vertices.len() as u32;

        vertices.push(Vertex::new(p1.x + tx, p1.y + ty, 0., 0., 0., colours[idx]));
        vertices.push(Vertex::new(p1.x - tx, p1.y - ty, 0., 0., 0., colours[idx]));
        vertices.push(Vertex::new(
            p2.x + tx,
            p2.y + ty,
            0.,
            0.,
            0.,
            colours[idx + 1],
        ));
        vertices.push(Vertex::new(
            p2.x - tx,
            p2.y - ty,
            0.,
            0.,
            0.,
            colours[idx + 1],
        ));

        indices.extend_from_slice(&[
            base.try_into().unwrap(),
            (base + 1).try_into().unwrap(),
            (base + 2).try_into().unwrap(),
            (base + 2).try_into().unwrap(),
            (base + 1).try_into().unwrap(),
            (base + 3).try_into().unwrap(),
        ]);
    }

    if vertices.is_empty() {
        return;
    }

    ctx.texture(None);
    ctx.draw_mode(DrawMode::Triangles);
    ctx.geometry(&vertices, &indices);
}
