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
use bitflags::Flags;

use crate::configuration::{
    ActiveTimesteps, AppExt, CommandsExt, DespawnSystems, SimulationMeta, Timestep,
    TimesteppedSystems,
};

#[derive(Resource)]
pub struct MouseCursorMeta {
    pub camera: Entity,
    pub spawn: SystemId<In<Timestep>>,
}

impl SimulationMeta for MouseCursorMeta {
    fn get(&self) -> (Entity, SystemId<In<Timestep>>) {
        (self.camera, self.spawn)
    }
}

#[derive(Resource)]
struct CursorMesh(Handle<Mesh>);

#[derive(Resource)]
struct CursorMaterial([Handle<StandardMaterial>; ActiveTimesteps::FLAGS.len()]);

fn material(timestep: Timestep) -> StandardMaterial {
    let colour = timestep.palette().sample_unchecked(0.0);

    StandardMaterial {
        base_color: colour.into(),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        ..default()
    }
}

#[derive(Component)]
struct Cursor;

const RENDER_LAYER: usize = 1;

struct Systems;

impl TimesteppedSystems for Systems {
    fn get_systems_for_timestep<T: Component>() -> ScheduleConfigs<ScheduleSystem> {
        move_cursor::<T>.into_configs()
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems_with_timestep::<Systems>();
}

fn setup(
    mut commands: Commands,
    mut despawns: ResMut<DespawnSystems>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const CURSOR_BOX_SIZE: f32 = 32.0;

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
            Projection::Orthographic(OrthographicProjection::default_3d()),
            Tonemapping::None,
            RenderLayers::layer(RENDER_LAYER),
        ))
        .id();

    commands.insert_resource(MouseCursorMeta { camera, spawn });

    let mesh = meshes.add(Rectangle::from_length(CURSOR_BOX_SIZE));
    commands.insert_resource(CursorMesh(mesh));

    let materials = [
        materials.add(material(Timestep::NoDelta)),
        materials.add(material(Timestep::VariableDelta)),
        materials.add(material(Timestep::SemiFixed)),
        materials.add(material(Timestep::Fixed)),
    ];
    commands.insert_resource(CursorMaterial(materials));
}

fn despawn(mut commands: Commands, cursors: Query<Entity, With<Cursor>>) {
    for entity in cursors.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(
    timestep: In<Timestep>,
    mut commands: Commands,
    mesh: Res<CursorMesh>,
    material: Res<CursorMaterial>,
) {
    commands.spawn_with_timestep(
        &timestep.0,
        (
            Cursor,
            Mesh3d(mesh.0.clone()),
            MeshMaterial3d(material.0[timestep.index()].clone()),
            RenderLayers::layer(RENDER_LAYER),
        ),
    );
}

fn move_cursor<T: Component>(
    mut cursor: Single<&mut Transform, (With<Cursor>, With<T>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };

    let position = camera.0.viewport_to_world_2d(camera.1, position).unwrap();
    cursor.translation = position.extend(-1.0);
}
