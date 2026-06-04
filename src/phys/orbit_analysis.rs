// Since SMA and SmA are unique vars this is really needed
#![allow(non_snake_case)]

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
}
