// Since SMA and SmA are unique vars this is really needed
#![allow(non_snake_case)]

use std::f64::consts::PI;

use macroquad::math::Vec2;
use physical_constants::NEWTONIAN_CONSTANT_OF_GRAVITATION;

use crate::objects::Objects;

#[derive(Debug)]
pub enum OrbitType {
    Circular,
    Elliptic,
    Parabolic,
    Hyperbolic,
}

impl OrbitType {
    /// Input domain: e: e > 0, e ∈ ℝ
    pub fn from_eccentricity(e: f32) -> Self {
        if e < 0.0 {
            panic!("Eccentricity cannot be negative");
        }

        let epsilon = 1e-5;

        if e < epsilon {
            OrbitType::Circular
        } else if (e - 1.0).abs() < epsilon {
            OrbitType::Parabolic
        } else if e < 1.0 {
            OrbitType::Elliptic
        } else {
            OrbitType::Hyperbolic
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            OrbitType::Circular => "Circular orbit",
            OrbitType::Elliptic => "Elliptic orbit",
            OrbitType::Parabolic => "Parabolic fly-by",
            OrbitType::Hyperbolic => "Hyperbolic fly-by",
        }
    }
}

#[derive(Debug)]
pub struct OrbitAnalysisResult {
    /// The primary is the body being orbited, deduced using osculating period
    /// The secondary is the body that is orbiting the primary, it is also the body that we are
    /// performing our analysis on
    /// NOTE: OrbitAnalsysisResult should be invalidated if the state (specifically SoA objects)
    /// changes
    pub primary: usize,
    pub secondary: usize,

    pub altitude: f32,
    pub orbital_speed: f32,
    pub escape_velocity: f32,
    pub circular_velocity: f32,
    pub speed_relative_to_circular: f32,
    pub speed_relative_to_escape: f32,

    /// Semi-Major Axis: Length of long side of the conic in metres
    pub SMA: f32,
    /// Semi-minor Axis: Length of the short side of the conic in metres
    pub SmA: f32,

    /// Osculating period
    pub period: f32,

    /// Apoapsis and periapsis are to centre of body, since all bodies are simulated as point
    /// masses. Therefore a radius isn't available
    pub apoapsis: Option<f32>,
    pub periapsis: f32,

    // Time to Ap or Pe
    pub ap_time: Option<f32>,
    pub pe_time: Option<f32>,

    /// ω
    pub argument_of_periapsis: f32,

    /// Eccentricity describes the shape of the conic
    /// e = 0: Circular orbit
    /// 0 < e < 1: Elliptic orbit
    /// e = 1: Parabolic fly-by
    /// e > 1: Hyperbolic fly-by
    pub eccentricity: f32,
    pub eccentricity_vector: Vec2,

    pub specific_energy: f32,
    pub specific_angular_momentum: f32,

    pub radial_velocity: f32,
    pub mean_motion: Option<f32>,

    pub orbit_type: OrbitType,

    /// The angular position (phase) of the secondary in its cycle
    pub true_anomaly: f32,

    /// Box containing position of points necessary to draw a kepler conic
    /// ConicAnnotations
    pub conic: Option<(Box<[Vec2]>, ConicAnnotations)>,

    /// Stores a set of tuples containing the osculating period of the secondary around each
    /// possible primary. The lowest one is used.
    // TODO: Is this really necessary?
    pub primary_candidate_scores: Box<[(usize, f32)]>,
}

#[derive(Debug)]
pub struct ConicAnnotations {
    pub apoapsis: Option<Vec2>,
    pub periapsis: Vec2,
}

/// This is used to get computed values from the osculating period finder into the main calculation
/// body
struct StageOneInfo {
    mu: f32,

    position_primary: Vec2,
    // velocity_primary: Vec2,
    // position_secondary: Vec2,
    // velocity_secondary: Vec2,
    relative_position: Vec2,
    relative_velocity: Vec2,

    mag_pos: f32,
    mag_vel: f32,

    orbital_energy: f32,

    SMA: f32,
    osculating_period: f32,
}

impl OrbitAnalysisResult {
    pub fn analyse_orbits(objects: &Objects, secondary: usize, kepler_conic: bool) -> Option<Self> {
        // Lower is better
        let mut best_score = f32::MAX;
        let mut best_primary = 0;

        // Tuple: Index, Osculating period
        let mut primaries: Vec<(usize, f32)> = Vec::with_capacity(objects.len() - 1);

        let mut s1: Option<StageOneInfo> = None;

        for primary in 0..objects.len() {
            if primary == secondary {
                continue;
            }

            let mu = NEWTONIAN_CONSTANT_OF_GRAVITATION as f32
                * (objects.mass[primary] + objects.mass[secondary]);

            let position_primary =
                Vec2::new(objects.position_x[primary], objects.position_y[primary]);
            let velocity_primary =
                Vec2::new(objects.velocity_x[primary], objects.velocity_y[primary]);
            let position_secondary =
                Vec2::new(objects.position_x[secondary], objects.position_y[secondary]);
            let velocity_secondary =
                Vec2::new(objects.velocity_x[secondary], objects.velocity_y[secondary]);

            let relative_position = position_secondary - position_primary;
            let relative_velocity = velocity_secondary - velocity_primary;

            let mag_pos = relative_position.length();
            let mag_vel = relative_velocity.length();

            let orbital_energy = ((mag_vel * mag_vel) / 2.0) - (mu / mag_pos);
            let SMA = -mu / (2.0 * orbital_energy);

            let osculating_period = 2.0 * PI as f32 * ((SMA * SMA * SMA) / mu).sqrt();

            let candidate_score = if orbital_energy < 0.0 {
                // Bias the calculation to strongly perfer bound orbits
                osculating_period / 100.0
            } else {
                mag_pos
            };

            if candidate_score < best_score {
                best_score = candidate_score;
                best_primary = primary;

                s1 = Some(StageOneInfo {
                    mu,
                    position_primary,
                    // velocity_primary,
                    // position_secondary,
                    // velocity_secondary,
                    relative_position,
                    relative_velocity,
                    mag_pos,
                    mag_vel,
                    orbital_energy,
                    SMA,
                    osculating_period,
                });
            }

            primaries.push((primary, osculating_period));
        }

        if let Some(s1) = s1 {
            let mag_angular_momentum = s1.relative_position.x * s1.relative_velocity.y
                - s1.relative_position.y * s1.relative_velocity.x;

            // Calculating eccentricity
            let eccentricity = Vec2::new(
                s1.relative_velocity.y * mag_angular_momentum / s1.mu,
                -s1.relative_velocity.x * mag_angular_momentum / s1.mu,
            ) - s1.relative_position.normalize();

            let mag_e = eccentricity.length();

            let SmA = if mag_e < 1.0 {
                s1.SMA * (1.0 - mag_e * mag_e).sqrt()
            } else {
                s1.SMA.abs() * (mag_e * mag_e - 1.0).sqrt()
            };

            let Ap = if mag_e < 1.0 {
                Some(s1.SMA * (1.0 + mag_e))
            } else {
                None
            };
            let Pe = s1.SMA * (1.0 - mag_e);

            let mut arg_pe = eccentricity.y.atan2(eccentricity.x);

            let cos_nu = eccentricity.dot(s1.relative_position) / (mag_e * s1.mag_pos);

            let mut true_anomaly = cos_nu.clamp(-1.0, 1.0).acos();

            if mag_e < 1e-5 {
                arg_pe = 0.0;
                true_anomaly = s1.relative_position.y.atan2(s1.relative_position.x);
            }

            if s1.relative_position.dot(s1.relative_velocity) < 0.0 {
                true_anomaly = 2.0 * std::f32::consts::PI - true_anomaly;
            }

            let mut time_to_ap = None;
            let mut time_to_pe = None;

            let mut mean_motion_out = None;

            // Calculating to to Ap and time to pe
            if mag_e < 1.0 {
                let eccentric_anomaly = 2.0
                    * (((1.0 - mag_e) / (1.0 + mag_e)).sqrt() * (true_anomaly / 2.0).tan()).atan();
                let mean_anomaly = eccentric_anomaly - mag_e * eccentric_anomaly.sin();
                let mean_motion = 2.0 * PI as f32 / s1.osculating_period;
                mean_motion_out = Some(mean_motion);
                let time_since_pe = mean_anomaly / mean_motion;
                time_to_pe =
                    Some((s1.osculating_period - time_since_pe).rem_euclid(s1.osculating_period));
                time_to_ap = if mag_e < 1.0 {
                    Some(if true_anomaly < PI as f32 {
                        (s1.osculating_period * 0.5 - time_since_pe)
                            .rem_euclid(s1.osculating_period)
                    } else {
                        1.5 * s1.osculating_period - time_since_pe
                    })
                } else {
                    None
                };
            }

            let radial_velocity = s1.relative_position.dot(s1.relative_velocity) / s1.mag_pos;

            // Maybe these are toy values than being useful orbital values, but they're cool
            // nontheless
            let escape_velocity = ((2.0 * s1.mu) / s1.mag_pos).sqrt();
            let circular_velocity = escape_velocity / 2.0f32.sqrt();
            let speed_relative_to_circular = s1.mag_vel / circular_velocity;
            let speed_relative_to_escape = s1.mag_vel / escape_velocity;

            let kepler_conic_data = if kepler_conic {
                Some(OrbitAnalysisResult::generate_conic_vertices(
                    s1.position_primary,
                    s1.SMA,
                    mag_e,
                    eccentricity,
                    arg_pe,
                    Ap,
                    Pe,
                ))
            } else {
                None
            };

            Some(OrbitAnalysisResult {
                primary: best_primary,
                secondary,
                SMA: s1.SMA,
                SmA,
                period: s1.osculating_period,
                apoapsis: Ap,
                periapsis: Pe,
                argument_of_periapsis: arg_pe,
                eccentricity: mag_e,
                true_anomaly,
                primary_candidate_scores: primaries.into(),
                ap_time: time_to_ap,
                pe_time: time_to_pe,
                specific_energy: s1.orbital_energy,
                specific_angular_momentum: mag_angular_momentum,
                orbit_type: OrbitType::from_eccentricity(mag_e),
                eccentricity_vector: eccentricity,
                radial_velocity,
                mean_motion: mean_motion_out,
                altitude: s1.mag_pos,
                orbital_speed: s1.mag_vel,
                escape_velocity,
                circular_velocity,
                speed_relative_to_circular,
                speed_relative_to_escape,
                conic: kepler_conic_data,
            })
        } else {
            None
        }
    }

    fn generate_conic_vertices(
        primary_pos: Vec2,
        SMA: f32,
        eccentricity: f32,
        eccentricity_vec: Vec2,
        argument_of_periapsis: f32,
        apoapsis: Option<f32>,
        periapsis: f32,
    ) -> (Box<[Vec2]>, ConicAnnotations) {
        const SEGMENTS: usize = 256;

        let mut points = vec![Vec2::ZERO; SEGMENTS].into_boxed_slice();

        for i in 0..SEGMENTS {
            let nu = if eccentricity <= 1.0 {
                i as f32 / SEGMENTS as f32 * std::f32::consts::TAU
            } else {
                let nu_max = (-(1.0 / eccentricity)).acos() - 0.05;
                -nu_max + (2.0 * nu_max) * i as f32 / SEGMENTS as f32
            };

            let cos_nu = nu.cos();
            let sin_nu = nu.sin();

            let p = SMA * (1.0 - eccentricity * eccentricity);
            let r = p / (1.0 + eccentricity * cos_nu);

            let x = r * cos_nu;
            let y = r * sin_nu;

            let cos_arg_pe = argument_of_periapsis.cos();
            let sin_arg_pe = argument_of_periapsis.sin();

            let x_rot = x * cos_arg_pe - y * sin_arg_pe;
            let y_rot = x * sin_arg_pe + y * cos_arg_pe;

            let world_x = primary_pos.x + x_rot;
            let world_y = primary_pos.y + y_rot;

            points[i] = Vec2::new(world_x, world_y);
        }

        let pe_direction = eccentricity_vec.normalize();

        let orbit_annotations = ConicAnnotations {
            apoapsis: if let Some(ap) = apoapsis {
                Some(primary_pos - pe_direction * ap)
            } else {
                None
            },
            periapsis: primary_pos + pe_direction * periapsis,
        };

        (points, orbit_annotations)
    }
}
