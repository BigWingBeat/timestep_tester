use bevy::{
    color::palettes::basic,
    ecs::system::SystemId,
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    render::view::RenderLayers,
};

use crate::timestep::{DespawnSystems, SimulationDescription, Timestep};

#[derive(Resource)]
pub struct SpawnMovingBox(pub SystemId<In<Timestep>>);

#[derive(Component)]
struct Box(Aabb2d);

#[derive(Component)]
#[require(Velocity)]
struct Movement {
    speed: f32,
    drag: f32,
}

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Resource, Default)]
enum Input {
    Left,
    #[default]
    None,
    Right,
}

impl Input {
    fn left(left: bool) -> Self {
        if left { Self::Left } else { Self::None }
    }

    fn right(right: bool) -> Self {
        if right { Self::Right } else { Self::None }
    }

    fn xor(left: bool, right: bool) -> Self {
        match (left, right) {
            (true, false) => Self::Left,
            (false, true) => Self::Right,
            _ => Self::None,
        }
    }
}

const RENDER_LAYER: usize = 1;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (handle_input, move_boxes, check_sensors).chain());
}

fn setup(mut commands: Commands, mut despawns: ResMut<DespawnSystems>) {
    let despawn = commands.register_system(despawn);
    despawns.0.push(despawn);
    let spawn = SpawnMovingBox(commands.register_system(spawn));
    commands.insert_resource(spawn);

    commands.spawn((Camera2d, RenderLayers::layer(RENDER_LAYER)));
    commands.init_resource::<Input>();
}

fn despawn(mut commands: Commands, boxes: Query<Entity, With<Box>>) {
    for entity in boxes.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(
    timestep: In<Timestep>,
    mut commands: Commands,
    mut description: Single<&mut TextSpan, With<SimulationDescription>>,
) {
    **description = "\n\nMoving box controls: 'A', 'Left', 'D', 'Right'".into();

    const BOX_SIZE: f32 = 75.0;
    const MOVING_BOX_SIZE: f32 = 70.0;

    const BASE_Y: f32 = -350.0;

    let y = ((timestep.0 as u8).ilog2() as f32 * BOX_SIZE) + BASE_Y;

    // Sensor boxes
    for i in -2..=2 {
        let i = i as f32;
        let pos = Vec2::new(BOX_SIZE * i * 3.0, y);
        commands.spawn((
            Box(Aabb2d::new(pos, Vec2::splat(BOX_SIZE / 2.0))),
            Transform::from_translation(pos.extend(0.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::RED, Vec2::splat(BOX_SIZE)),
        ));
    }

    // Moving box
    commands.spawn((
        Movement {
            speed: 1000.0,
            drag: 100.0,
        },
        Box(Aabb2d::new(
            Vec2::new(0.0, y),
            Vec2::splat(MOVING_BOX_SIZE / 2.0),
        )),
        Transform::from_translation(Vec3::new(0.0, y, 0.0)),
        RenderLayers::layer(RENDER_LAYER),
        Sprite::from_color(basic::LIME, Vec2::splat(MOVING_BOX_SIZE)),
    ));
}

fn handle_input(mut input: ResMut<Input>, keys: Res<ButtonInput<KeyCode>>) {
    let left = keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);

    match &*input {
        Input::Left if !left => *input = Input::right(right),
        Input::Right if !right => *input = Input::left(left),
        Input::None => *input = Input::xor(left, right),
        _ => {}
    }
}

fn move_boxes(
    mut movers: Query<(&mut Transform, &mut Box, &mut Velocity, &Movement)>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (mut transform, mut aabb, mut velocity, configuration) in movers.iter_mut() {
        match *input {
            Input::Left => velocity.0 = -configuration.speed * time.delta_secs(),
            Input::None => velocity
                .0
                .smooth_nudge(&0.0, configuration.drag, time.delta_secs()),
            Input::Right => velocity.0 = configuration.speed * time.delta_secs(),
        }

        let mut_x = |x: &mut f32| {
            *x += velocity.0;
            if x.abs() > 600.0 {
                *x = -x.clamp(-600.0, 600.0);
            }
        };

        mut_x(&mut transform.translation.x);
        mut_x(&mut aabb.0.min.x);
        mut_x(&mut aabb.0.max.x);
    }
}

fn check_sensors(
    movers: Query<&Box, With<Movement>>,
    mut sensors: Query<(&Box, &mut Sprite), Without<Movement>>,
) {
    for (sensor_aabb, mut sprite) in sensors.iter_mut() {
        if movers.iter().any(|aabb| sensor_aabb.0.intersects(&aabb.0)) {
            sprite.color = basic::BLUE.into();
        } else {
            sprite.color = basic::RED.into();
        }
    }
}
