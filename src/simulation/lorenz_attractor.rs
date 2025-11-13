use bevy::{
    camera::visibility::RenderLayers,
    color::ColorCurve,
    core_pipeline::tonemapping::Tonemapping,
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    math::DVec3,
    prelude::*,
};

use crate::configuration::{
    AppExt, CommandsExt, DespawnSystems, SimulationMeta, Timestep, TimesteppedSystems,
};

#[derive(Resource)]
pub struct LorenzAttractorMeta {
    pub camera: Entity,
    pub spawn: SystemId<In<Timestep>>,
}

impl SimulationMeta for LorenzAttractorMeta {
    fn get(&self) -> (Entity, SystemId<In<Timestep>>) {
        (self.camera, self.spawn)
    }
}

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
    curve: ColorCurve<Oklaba>,
    interp_seconds: f32,
    factor: f32,
    seq: Vec<Oklaba>,
}

impl Colours {
    fn new(curve: ColorCurve<Oklaba>, interp_seconds: f32) -> Self {
        Self {
            curve,
            interp_seconds,
            factor: 0.0,
            seq: Vec::new(),
        }
    }

    fn from_timestep(timestep: &Timestep) -> Self {
        const INTERP_SECONDS: f32 = 10.0;
        Self::new(timestep.palette(), INTERP_SECONDS)
    }

    fn push_next(&mut self, dt: f32) {
        self.factor += (self.curve.domain().length() / self.interp_seconds) * dt;
        if !self.curve.domain().contains(self.factor) {
            self.factor = self.curve.domain().start();
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

    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                is_active: false,
                ..default()
            },
            Tonemapping::None,
            RenderLayers::layer(RENDER_LAYER),
            Transform::from_xyz(-100.0, 150.0, 150.0)
                .looking_at(Vec3::new(0.0, 30.0, 0.0), Vec3::Y),
        ))
        .id();

    commands.insert_resource(LorenzAttractorMeta { camera, spawn });

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

fn spawn(timestep: In<Timestep>, mut commands: Commands) {
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
