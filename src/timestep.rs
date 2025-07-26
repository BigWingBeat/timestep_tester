use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn).add_systems(Update, run);
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "Moving box controls: 'A', 'Left', 'D', 'Right'
Timestep toggles:
1: No Delta Time
2: Variable Delta Time
3: Semi-Fixed Timestep
4: Fixed Timestep
Cursor Colours:
YELLOW: Window::cursor_position & Camera::viewport_to_world_2d
AQUA: EventReader<CursorMoved>::position & Camera::viewport_to_world_2d
FUCHSIA: EventReader<CursorMoved>::delta & Vec2::reflect(Vec2::Y)
WHITE: EventReader<MouseMotion>::delta & Vec2::reflect(Vec2::Y)
BLACK: Res<AccumulatedMouseMotion>::delta & Vec2::reflect(Vec2::Y)",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn run(keys: Res<ButtonInput<KeyCode>>) {}
