use bevy::{
    camera::visibility::RenderLayers,
    core_pipeline::tonemapping::Tonemapping,
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::configuration::{
    ActiveTimesteps, AppExt, CommandsExt, DespawnSystems, SimulationMeta, Timestep,
    TimestepComponent, TimesteppedSystems,
};

#[derive(Resource)]
pub struct MovingBarsMeta {
    pub camera: Entity,
    pub spawn: SystemId<In<Timestep>>,
}

impl SimulationMeta for MovingBarsMeta {
    fn get(&self) -> (Entity, SystemId<In<Timestep>>) {
        (self.camera, self.spawn)
    }
}

#[derive(Component)]
struct Bar;

const MAX_X: f32 = 800.0;
const MOVE_SPEED: f32 = 200.0;

const RENDER_LAYER: usize = 2;

struct Systems;

impl TimesteppedSystems for Systems {
    fn get_systems_for_timestep<T: TimestepComponent>() -> ScheduleConfigs<ScheduleSystem> {
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
            Camera2d,
            Camera {
                is_active: false,
                ..default()
            },
            Tonemapping::None,
            RenderLayers::layer(RENDER_LAYER),
        ))
        .id();
    commands.insert_resource(MovingBarsMeta { camera, spawn });
}

fn despawn(mut commands: Commands, bars: Query<Entity, With<Bar>>) {
    for entity in bars.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(timestep: In<Timestep>, mut commands: Commands) {
    const WIDTH: f32 = 80.0;

    let colour = timestep.palette().sample_unchecked(0.0);

    let count = (MAX_X / (WIDTH * 2.0)) as u8;
    for i in 0..count {
        // The resize_y system handles setting y pos and height, so we don't bother duplicating that work here
        let x = (i as f32) * WIDTH * 2.0;
        commands.spawn_with_timestep(
            &timestep.0,
            (
                Transform::from_xyz(x, 0.0, 1.0),
                RenderLayers::layer(RENDER_LAYER),
                Sprite::from_color(colour, Vec2::splat(WIDTH)),
                Bar,
            ),
        );
    }
}

fn run<T: TimestepComponent>(
    mut bars: Query<(&mut Transform, &mut Sprite), (With<Bar>, With<T>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    active_timesteps: Res<ActiveTimesteps>,
    time: Res<Time>,
) {
    let window_height = window.height();

    let total = active_timesteps.bits().count_ones();
    let height = window_height / (total as f32);

    let offset = (window_height / 2.0) - (height / 2.0);
    let above_count = (active_timesteps.bits() & ((T::TIMESTEP as u8) - 1)).count_ones();
    let y = offset - ((above_count as f32) * height);

    for (mut transform, mut sprite) in bars.iter_mut() {
        let new_x = transform.translation.x + (MOVE_SPEED * time.delta_secs());
        transform.translation.x = new_x % MAX_X;
        transform.translation.y = y;
        sprite.custom_size.as_mut().unwrap().y = height;
    }
}
