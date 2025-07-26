use bevy::{
    color::palettes::basic,
    ecs::system::SystemId,
    input::mouse::{AccumulatedMouseMotion, MouseMotion},
    prelude::*,
    render::view::RenderLayers,
    window::PrimaryWindow,
};

use crate::configuration::{DespawnSystems, SimulationDescription, Timestep};

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

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup).add_systems(
        Update,
        (
            queue_initialise_deltas,
            window_cursor_position,
            cursor_moved_event,
            mouse_motion_event,
            accumulated_mouse_motion,
        ),
    );
}

fn setup(mut commands: Commands, mut despawns: ResMut<DespawnSystems>) {
    let despawn = commands.register_system(despawn);
    despawns.0.push(despawn);
    let spawn = SpawnMouseCursor(commands.register_system(spawn));
    commands.insert_resource(spawn);
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
    **description = "\n\nCursor Colours:
YELLOW: Window::cursor_position & Camera::viewport_to_world_2d
AQUA: EventReader<CursorMoved>::position & Camera::viewport_to_world_2d
FUCHSIA: EventReader<CursorMoved>::delta & Vec2::reflect(Vec2::Y)
WHITE: EventReader<MouseMotion>::delta & Vec2::reflect(Vec2::Y)
BLACK: Res<AccumulatedMouseMotion>::delta & Vec2::reflect(Vec2::Y)"
        .into();

    const RENDER_LAYER: usize = 1;

    const CURSOR_BOX_SIZE: f32 = 16.0;

    commands.spawn((
        WindowCursorPosition,
        Offset(Vec2::new(8.0, 8.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::YELLOW, Vec2::splat(CURSOR_BOX_SIZE)),
    ));

    commands.spawn((
        CursorMovedEventPosition,
        Offset(Vec2::new(-8.0, 8.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::AQUA, Vec2::splat(CURSOR_BOX_SIZE)),
    ));

    commands.spawn((
        CursorMovedEventDelta,
        Offset(Vec2::new(8.0, -8.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::FUCHSIA, Vec2::splat(CURSOR_BOX_SIZE)),
    ));

    commands.spawn((
        MouseMotionEvent,
        Offset(Vec2::new(-8.0, -8.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::WHITE, Vec2::splat(CURSOR_BOX_SIZE)),
    ));

    commands.spawn((
        AccumulatedMouseMotionRes,
        Offset(Vec2::new(-24.0, -24.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::BLACK, Vec2::splat(CURSOR_BOX_SIZE)),
    ));
}

fn queue_initialise_deltas(mut commands: Commands, mut events: EventReader<CursorMoved>) {
    let something_frame = events.read().any(|event| event.delta.is_some());
    if something_frame {
        commands.run_system_cached(initialise_deltas);
    }
}

fn initialise_deltas(
    mut cursors: Query<(&mut Transform, &Offset, &mut DeltaInitialised)>,
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

fn window_cursor_position(
    mut cursor: Single<(&mut Transform, &Offset), With<WindowCursorPosition>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };

    let position = camera.0.viewport_to_world_2d(camera.1, position).unwrap();
    cursor.0.translation = (position + cursor.1.0).extend(Z);
}

fn cursor_moved_event(
    mut cursor_pos: Single<
        (&mut Transform, &Offset),
        (
            With<CursorMovedEventPosition>,
            Without<CursorMovedEventDelta>,
        ),
    >,
    mut cursor_delta: Single<
        (&mut Transform, &Offset),
        (
            With<CursorMovedEventDelta>,
            Without<CursorMovedEventPosition>,
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
        } else {
            // cursor_delta.0.translation = (position + cursor_delta.1.0).extend(0.0);
        }
    }
}

fn mouse_motion_event(
    mut cursor: Single<(&mut Transform, &Offset), With<MouseMotionEvent>>,
    mut events: EventReader<MouseMotion>,
) {
    for event in events.read() {
        cursor.0.translation += event.delta.reflect(Vec2::Y).extend(0.0) * 2.0;
    }
}

fn accumulated_mouse_motion(
    mut cursor: Single<(&mut Transform, &Offset), With<AccumulatedMouseMotionRes>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    cursor.0.translation += accumulated_mouse_motion.delta.reflect(Vec2::Y).extend(0.0) * 2.0;
}
