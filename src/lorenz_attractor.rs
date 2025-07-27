use bevy::{
    color::{ColorCurve, palettes::tailwind},
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    math::DVec3,
    prelude::*,
    render::view::RenderLayers,
};

use crate::configuration::{
    AppExt, CommandsExt, DespawnSystems, SimulationDescription, Timestep, TimesteppedSystems,
};

#[derive(Resource)]
pub struct SpawnLorenzAttractor(pub SystemId<In<Timestep>>);

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

    fn from_timestep(timestep: &Timestep) -> Self {
        const INTERP_SECONDS: f32 = 10.0;
        let colours = match timestep {
            Timestep::NoDelta => [
                tailwind::ROSE_500.into(),
                tailwind::ROSE_900.into(),
                tailwind::RED_900.into(),
            ],
            Timestep::VariableDelta => [
                tailwind::AMBER_500.into(),
                tailwind::AMBER_900.into(),
                tailwind::YELLOW_900.into(),
            ],
            Timestep::SemiFixed => [
                tailwind::SKY_500.into(),
                tailwind::SKY_900.into(),
                tailwind::BLUE_900.into(),
            ],
            Timestep::Fixed => [
                tailwind::GREEN_500.into(),
                tailwind::GREEN_900.into(),
                tailwind::EMERALD_900.into(),
            ],
        };
        Self::new(colours, INTERP_SECONDS)
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

const RENDER_LAYER: usize = 0;

struct Systems;

impl TimesteppedSystems for Systems {
    fn get_systems_for_timestep<T: Component>() -> ScheduleConfigs<ScheduleSystem> {
        run::<T>.into_configs()
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems_with_timestep::<Systems>();
}

fn setup(mut commands: Commands, mut despawns: ResMut<DespawnSystems>) {
    let despawn = commands.register_system(despawn);
    despawns.0.push(despawn);
    let spawn = commands.register_system(spawn);
    commands.insert_resource(SpawnLorenzAttractor(spawn));

    commands.spawn((
        Camera3d::default(),
        RenderLayers::layer(RENDER_LAYER),
        Transform::from_xyz(-100.0, 150.0, 150.0).looking_at(Vec3::new(0.0, 30.0, 0.0), Vec3::Y),
    ));

    commands.insert_resource(Parameters {
        sigma: 10.0,
        rho: 28.0,
        beta: 8.0 / 3.0,
    });
}

fn despawn(mut commands: Commands, trajectories: Query<Entity, With<Trajectory>>) {
    for entity in trajectories.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(
    timestep: In<Timestep>,
    mut commands: Commands,
    mut description: Single<&mut TextSpan, With<SimulationDescription>>,
) {
    **description = "Lorenz Attractor".into();

    commands.spawn_with_timestep(
        &timestep.0,
        (
            Trajectory(DVec3::new(2.0, 1.0, 1.0)),
            RenderLayers::layer(RENDER_LAYER),
            Colours::from_timestep(&timestep.0),
        ),
    );
}

fn run<T: Component>(
    mut trajectories: Query<(&mut Trajectory, &mut Points, &mut Colours), With<T>>,
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
