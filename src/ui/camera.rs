use macroquad::{
    camera::Camera2D,
    input::{KeyCode, MouseButton, is_key_down, is_mouse_button_down, mouse_position, mouse_wheel},
    math::{Vec2, vec2},
    window::{screen_height, screen_width},
};

pub struct CameraController {
    pub camera: Camera2D,
    zoom_level: f32,
    dragging: bool,
    last_mouse: Vec2,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            camera: Camera2D {
                zoom: vec2(2.0 / screen_width() * 200.0, -2.0 / screen_height() * 200.0),
                target: vec2(0.0, 0.0),
                ..Default::default()
            },
            // x zoom
            zoom_level: 2.0 / screen_width() * 200.0,
            dragging: false,
            last_mouse: mouse_position().into(),
        }
    }

    pub fn update(&mut self) {
        let mouse: Vec2 = mouse_position().into();

        // Middle mouse OR Alt + Left Mouse
        let drag_active = is_mouse_button_down(MouseButton::Middle)
            || (is_key_down(KeyCode::LeftAlt) && is_mouse_button_down(MouseButton::Left));

        // Start drag
        if drag_active && !self.dragging {
            self.dragging = true;
            self.last_mouse = mouse;
        }

        // End drag
        if !drag_active {
            self.dragging = false;
        }

        // Pan camera
        if self.dragging {
            let mut delta = mouse - self.last_mouse;

            // Flip y axis movement
            delta.y = -delta.y;

            // Convert screen movement into world movement
            let zoom_scale = vec2(2.0 / screen_width(), 2.0 / screen_height());

            self.camera.target -= delta * zoom_scale / self.camera.zoom.abs();

            self.last_mouse = mouse;
        }
        // Scroll zoom
        let (_, scroll_y) = mouse_wheel();

        if scroll_y != 0.0 {
            // Mouse position in screen space
            let mouse_screen: Vec2 = mouse_position().into();

            // World position BEFORE zoom
            let world_before = self.camera.screen_to_world(mouse_screen);

            // Zoom factor
            let zoom_factor = if scroll_y > 0.0 { 1.1 } else { 0.9 };

            // Apply zoom
            self.zoom_level *= zoom_factor;

            // Clamp zoom
            self.zoom_level = self.zoom_level.clamp(0.0005, 10.0);

            // Compute zoom to work out where to move to
            self.compute_zoom();

            // World position AFTER zoom
            let world_after = self.camera.screen_to_world(mouse_screen);

            // Move camera target so cursor stays fixed on same world point
            self.camera.target += world_before - world_after;
        }

        self.compute_zoom();
    }

    fn compute_zoom(&mut self) {
        self.camera.zoom.x = self.zoom_level;
        self.camera.zoom.y = self.zoom_level * -(screen_width() / screen_height());
    }
}
