use macroquad::{prelude::*, ui::{hash, root_ui, widgets}};
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

// All masses are singularities
#[derive(Debug, Clone, Copy)]
struct Object {
    position: Vec2, // ms^-1
    velocity: Vec2, // ms^-1
    mass: f32,      // kg
}

impl Default for Object {
    fn default() -> Self {
        Self {
            position: Vec2::new(2.0, 2.0),
            velocity: Vec2::new(0.0, 0.0),
            mass: 1.0, // 1 blistering kilogram
        }
    }
}

enum Mode {
    Simulating,
    Paused,
}

impl Mode {
    fn status(&self) -> String {
        match &self {
            Mode::Simulating => "SIMULATING".to_string(),
            Mode::Paused => "PAUSED".to_string(),
        }
    }
}

fn physics_step(object_vector: &mut Vec<Object>, ut: &mut f32, time_multiplier: f32) {
    let vector_state = object_vector.clone();

    for object in object_vector.iter_mut().enumerate() {
        let index = object.0;
        let object = object.1;

        // Apply forces (and their accelerations)
        // |F| = m_1 |a|
        // |F| = G(m_1 m_2) / r^2
        // r = sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)
        // We want this in terms of a
        // |a| = G(m_2) / r^2
        // |a| = G(m_2) / (m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2
        // Since the vector a will point towards m_2
        // Get unit vector pointing towards the second mass
        // butt (vec[2]) = [(m_1.x - m_2.x)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2), (m_1.y - m_2.y)/sqrt((m_1.x - m_2.x)^2 + (m_1.y - m_2.y)^2)];
        // Multiply it by the vector a to get the acceleration vector
        // a = |a| * butt
        // v = v += a * dt

        let mut a = Vec2::new(0.0, 0.0);

        for object_two in vector_state.iter() {
            if vector_state[index].position == object_two.position {
                continue;
            }

            let r_squared = (object.position.x as f64 - object_two.position.x as f64).powi(2)
                + (object.position.y as f64 - object_two.position.y as f64).powi(2);

            let mag_a: f64 =
                (NEWTONIAN_CONSTANT_OF_GRAVITATION * object_two.mass as f64) / r_squared;
            a += Vec2::new(
                -mag_a as f32 * (object.position.x - object_two.position.x)
                    / r_squared.sqrt() as f32,
                -mag_a as f32 * (object.position.y - object_two.position.y)
                    / r_squared.sqrt() as f32,
            );
        }

        // Make velocity velocit
        object.position += object.velocity * get_frame_time() * time_multiplier;
        // Apply the acceleration to the velocity
        object.velocity += a * get_frame_time() * time_multiplier;
    }

    *ut += get_frame_time() * time_multiplier;
}

fn draw_state(object_vector: &mut Vec<Object>, zoom: f32, position: Vec2) {
    for object in object_vector {
        // Draw it
        draw_circle(
            object.position.x * zoom + position.x,
            object.position.y * zoom + position.y,
            5.0,
            BLACK,
        ); // Draw a thing
    }
}

#[macroquad::main("n-body")]
async fn main() {
    let mut object_vector: Vec<Object> = vec![];
    let mut ut: f32 = 0.0;

    // Create a mass
    // object_vector.push(Object { velocity: Vec2::new(0.0, 0.02), ..Default::default() } );

    // Create another mass
    // object_vector.push(Object { position: Vec2::new(2.1, 2.0), velocity: Vec2::new(0.0, 0.0), mass: 1000000.0 });

    // Three body problem
    object_vector.push(Object {
        position: Vec2::new(2.0, 2.0),
        velocity: Vec2::new(0.045, 0.005),
        mass: 1000000.0,
    });
    object_vector.push(Object {
        position: Vec2::new(2.1, 2.1),
        velocity: Vec2::new(0.0, 0.01),
        mass: 1000000.0,
    });
    object_vector.push(Object {
        position: Vec2::new(2.1, 2.2),
        velocity: Vec2::new(0.01, -0.018),
        mass: 1000000.0,
    });
    object_vector.push(Object {
        position: Vec2::new(2.1, 2.25),
        velocity: Vec2::new(0.03, -0.020),
        mass: 1000000.0,
    });

    let mut mode = Mode::Paused;
    let mut time_multiplier = 1.0;

    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        widgets::Window::new(hash!(), vec2(200.0, 200.0), vec2(542.0, 430.0))
        .label("Simulation Control")
        .titlebar(true)
        .ui(&mut root_ui(), |ui| {
            ui.label(Vec2::new(10.0, 50.0), "Status:");
            if ui.button(Vec2::new(10.0, 70.0), mode.status()) {
                match mode {
                    Mode::Simulating => mode = Mode::Paused,
                    Mode::Paused => mode = Mode::Simulating,
                }
            }
            let tw_range = -1f32..4f32;
            ui.slider(hash!(), "Time warp (-1x, 4x)", tw_range.clone(), &mut time_multiplier);
        });

        match mode {
            Mode::Simulating => physics_step(&mut object_vector, &mut ut, time_multiplier),
            Mode::Paused => {},
        }

        draw_state(&mut object_vector, 1000.0, Vec2::new(-1900.0, -1900.0));

        draw_text(
            &format! {"dt: {}", get_frame_time()}.to_string(),
            20.0,
            20.0,
            30.0,
            DARKGRAY,
        );
        draw_text(
            &format! {"ut: {}", ut}.to_string(),
            20.0,
            50.0,
            30.0,
            DARKGRAY,
        );

        next_frame().await;

        // std::process::exit(1);
    }
}
