use bevy::{
    color::palettes::basic,
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    input::mouse::{AccumulatedMouseMotion, MouseMotion},
    prelude::*,
    render::view::RenderLayers,
    window::PrimaryWindow,
};

use crate::configuration::{
    AppExt, CommandsExt, DespawnSystems, SimulationDescription, Timestep, TimesteppedSystems,
};

#[derive(Resource)]
pub struct SpawnMouseCursor(pub SystemId<In<Timestep>>);

#[derive(Component)]
struct WindowCursorPosition;

#[derive(Component)]
struct CursorMovedEventPosition;

#[derive(Component)]
#[require(DeltaInitialised)]
struct CursorMovedEventDelta;

#[derive(Component)]
#[require(DeltaInitialised)]
struct MouseMotionEvent;

#[derive(Component)]
#[require(DeltaInitialised)]
struct AccumulatedMouseMotionRes;

#[derive(Component, Default)]
struct DeltaInitialised(bool);

#[derive(Component)]
struct Offset(Vec2);

const Z: f32 = 1.0;

struct Systems;

impl TimesteppedSystems for Systems {
    fn get_systems_for_timestep<T: Component>() -> ScheduleConfigs<ScheduleSystem> {
        (
            queue_initialise_deltas::<T>,
            window_cursor_position::<T>,
            cursor_moved_event::<T>,
            mouse_motion_event::<T>,
            accumulated_mouse_motion::<T>,
        )
            .into_configs()
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
    commands.insert_resource(SpawnMouseCursor(spawn));
}

fn despawn(mut commands: Commands, cursors: Query<Entity, With<Offset>>) {
    for entity in cursors.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(
    timestep: In<Timestep>,
    mut commands: Commands,
    mut description: Single<&mut TextSpan, With<SimulationDescription>>,
) {
    **description = "Cursor Colours:
YELLOW: Window::cursor_position & Camera::viewport_to_world_2d
AQUA: EventReader<CursorMoved>::position & Camera::viewport_to_world_2d
FUCHSIA: EventReader<CursorMoved>::delta & Vec2::reflect(Vec2::Y)
WHITE: EventReader<MouseMotion>::delta & Vec2::reflect(Vec2::Y)
BLACK: Res<AccumulatedMouseMotion>::delta & Vec2::reflect(Vec2::Y)"
        .into();

    const RENDER_LAYER: usize = 1;

    const CURSOR_BOX_SIZE: f32 = 16.0;

    commands.spawn_with_timestep(
        &timestep.0,
        (
            WindowCursorPosition,
            Offset(Vec2::new(8.0, 8.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::YELLOW, Vec2::splat(CURSOR_BOX_SIZE)),
        ),
    );

    commands.spawn_with_timestep(
        &timestep.0,
        (
            CursorMovedEventPosition,
            Offset(Vec2::new(-8.0, 8.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::AQUA, Vec2::splat(CURSOR_BOX_SIZE)),
        ),
    );

    commands.spawn_with_timestep(
        &timestep.0,
        (
            CursorMovedEventDelta,
            Offset(Vec2::new(8.0, -8.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::FUCHSIA, Vec2::splat(CURSOR_BOX_SIZE)),
        ),
    );

    commands.spawn_with_timestep(
        &timestep.0,
        (
            MouseMotionEvent,
            Offset(Vec2::new(-8.0, -8.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::WHITE, Vec2::splat(CURSOR_BOX_SIZE)),
        ),
    );

    commands.spawn_with_timestep(
        &timestep.0,
        (
            AccumulatedMouseMotionRes,
            Offset(Vec2::new(-24.0, -24.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::BLACK, Vec2::splat(CURSOR_BOX_SIZE)),
        ),
    );
}

fn queue_initialise_deltas<T: Component>(
    mut commands: Commands,
    mut events: EventReader<CursorMoved>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if window.cursor_position().is_none() {
        return;
    }

    let something_frame = events.read().any(|event| event.delta.is_some());
    if something_frame {
        commands.run_system_cached(initialise_deltas::<T>);
    }
}

fn initialise_deltas<T: Component>(
    mut cursors: Query<(&mut Transform, &Offset, &mut DeltaInitialised), With<T>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let position = window.cursor_position().unwrap();
    let position = camera.0.viewport_to_world_2d(camera.1, position).unwrap();

    for (mut cursor, offset, mut init) in cursors.iter_mut() {
        if !init.0 {
            cursor.translation = (position + offset.0).extend(Z);
            init.0 = true;
        }
    }
}

fn window_cursor_position<T: Component>(
    mut cursor: Single<(&mut Transform, &Offset), (With<WindowCursorPosition>, With<T>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };

    let position = camera.0.viewport_to_world_2d(camera.1, position).unwrap();
    cursor.0.translation = (position + cursor.1.0).extend(Z);
}

fn cursor_moved_event<T: Component>(
    mut cursor_pos: Single<
        (&mut Transform, &Offset),
        (
            With<CursorMovedEventPosition>,
            Without<CursorMovedEventDelta>,
            With<T>,
        ),
    >,
    mut cursor_delta: Single<
        (&mut Transform, &Offset),
        (
            With<CursorMovedEventDelta>,
            Without<CursorMovedEventPosition>,
            With<T>,
        ),
    >,
    mut events: EventReader<CursorMoved>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    for event in events.read() {
        let position = camera
            .0
            .viewport_to_world_2d(camera.1, event.position)
            .unwrap();
        cursor_pos.0.translation = (position + cursor_pos.1.0).extend(Z);

        if let Some(delta) = event.delta {
            cursor_delta.0.translation += delta.reflect(Vec2::Y).extend(0.0);
        }
    }
}

fn mouse_motion_event<T: Component>(
    mut cursor: Single<(&mut Transform, &Offset), (With<MouseMotionEvent>, With<T>)>,
    mut events: EventReader<MouseMotion>,
) {
    for event in events.read() {
        cursor.0.translation += event.delta.reflect(Vec2::Y).extend(0.0) * 2.0;
    }
}

fn accumulated_mouse_motion<T: Component>(
    mut cursor: Single<(&mut Transform, &Offset), (With<AccumulatedMouseMotionRes>, With<T>)>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    cursor.0.translation += accumulated_mouse_motion.delta.reflect(Vec2::Y).extend(0.0) * 2.0;
}
