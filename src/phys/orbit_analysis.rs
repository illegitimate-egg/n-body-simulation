// Since SMA and SmA are unique vars this is really needed
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::f64::consts::PI;

use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use crate::objects::Objects;

pub struct OrbitAnalysisResult {
    /// The primary is the body being orbited, deduced using osculating period
    /// The secondary is the body that is orbiting the primary, it is also the body that we are
    /// performing our analysis on
    /// NOTE: OrbitAnalsysisResult should be invalidated if the state (specifically SoA objects)
    /// changes
    primary: usize,
    secondary: usize,

    /// Semi-Major Axis: Length of long side of the conic in metres
    SMA: f32,
    /// Semi-minor Axis: Length of the short side of the conic in metres
    SmA: f32,

    /// Apoapsis and periapsis are to centre of body, since all bodies are simulated as point
    /// masses. Therefore a radius isn't available
    apoapsis: f32,
    periapsis: f32,

    /// ω
    argument_of_periapsis: f32,

    /// Eccentricity describes the shape of the conic
    /// e = 0: Circular orbit
    /// 0 < e < 1: Elliptic orbit
    /// e = 1: Parabolic fly-by
    /// e > 1: Hyperbolic fly-by
    eccentricity: f32,

    /// The angular position (phase) of the secondary in its cycle
    true_anomaly: f32,

    /// Stores a set of tuples containing the osculating period of the secondary around each
    /// possible primary. The lowest one is used.
    /// TODO: Is this really necessary?
    primary_candidate_scores: Box<[(usize, f32)]>,
}

impl OrbitAnalysisResult {
    // pub fn analyse_orbits(objects: &Objects, secondary: usize) -> Self {}

    pub fn osculating_period(objects: &Objects, primary: usize, secondary: usize) -> Option<f32> {
        let mu = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32
            * (objects.mass[primary] + objects.mass[secondary]);

        let position_primary = Vec2::new(objects.position_x[primary], objects.position_y[primary]);
        let velocity_primary = Vec2::new(objects.velocity_x[primary], objects.velocity_y[primary]);
        let position_secondary =
            Vec2::new(objects.position_x[secondary], objects.position_y[secondary]);
        let velocity_secondary =
            Vec2::new(objects.velocity_x[secondary], objects.velocity_y[secondary]);

        let relative_position = position_secondary - position_primary;
        let relative_velocity = velocity_secondary - velocity_primary;

        let mag_pos = relative_position.length();
        let mag_vel = relative_velocity.length();

        let mag_angular_momentum =
            relative_position.x * relative_velocity.y - relative_position.y * relative_velocity.x;

        let orbital_energy = ((mag_vel * mag_vel) / 2.0) - (mu / mag_pos);
        let SMA = -mu / (2.0 * orbital_energy);

        if orbital_energy >= 0.0 {
            return None;
        }

        Some(2.0 * PI as f32 * ((SMA * SMA * SMA) / mu).sqrt())
    }
}
