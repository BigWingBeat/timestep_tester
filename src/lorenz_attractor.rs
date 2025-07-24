use bevy::{
    color::{ColorCurve, palettes::tailwind},
    math::DVec3,
    prelude::*,
};

#[derive(Resource)]
struct Parameters {
    /// σ
    sigma: f64,
    /// ρ
    rho: f64,
    /// β
    beta: f64,
}

#[derive(Component)]
#[require(Points)]
struct Trajectory(DVec3);

#[derive(Component, Default)]
struct Points(Vec<Vec3>);

#[derive(Component)]
struct Colours {
    curve: ColorCurve<Oklcha>,
    interp_seconds: f32,
    factor: f32,
    seq: Vec<Oklcha>,
}

impl Colours {
    fn new(colours: impl IntoIterator<Item = Oklcha>, interp_seconds: f32) -> Self {
        Self {
            curve: ColorCurve::new(colours).unwrap(),
            interp_seconds,
            factor: 0.0,
            seq: Vec::new(),
        }
    }

    fn push_next(&mut self, dt: f32) {
        self.factor += (self.curve.domain().end() / self.interp_seconds) * dt;
        let new_factor = self.curve.domain().clamp(self.factor);
        if new_factor != self.factor {
            self.factor = new_factor;
            self.interp_seconds = -self.interp_seconds;
        }
        let colour = self.curve.sample_unchecked(self.factor);
        self.seq.push(colour);
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn).add_systems(Update, run);
}

fn spawn(mut commands: Commands) {
    commands.insert_resource(Parameters {
        sigma: 10.0,
        rho: 28.0,
        beta: 8.0 / 3.0,
    });

    commands.spawn((
        Trajectory(DVec3::new(2.0, 1.0, 1.0)),
        Colours::new(
            [
                tailwind::SKY_500.into(),
                tailwind::SKY_900.into(),
                tailwind::BLUE_900.into(),
            ],
            10.0,
        ),
    ));
}

fn run(
    mut trajectories: Query<(&mut Trajectory, &mut Points, &mut Colours)>,
    mut gizmos: Gizmos,
    parameters: Res<Parameters>,
    time: Res<Time>,
) {
    let Parameters { sigma, rho, beta } = *parameters;
    for (mut trajectory, mut points, mut colours) in trajectories.iter_mut() {
        let pos = &mut trajectory.0;
        let delta = DVec3::new(
            pos.z * (rho - pos.y) - pos.x,
            pos.z * pos.x - beta * pos.y,
            sigma * (pos.x - pos.z),
        );
        *pos += delta * time.delta_secs_f64();

        let pos = pos.as_vec3();
        points.0.push(pos);
        colours.push_next(time.delta_secs());
        gizmos.linestrip_gradient(points.0.iter().copied().zip(colours.seq.iter().copied()));
    }
}
