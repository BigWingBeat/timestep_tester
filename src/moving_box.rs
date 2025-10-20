use bevy::{
    camera::visibility::RenderLayers,
    color::palettes::basic,
    core_pipeline::tonemapping::Tonemapping,
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
};

use crate::configuration::{
    AppExt, CommandsExt, DespawnSystems, SimulationDescription, SimulationMeta, Timestep,
    TimesteppedSystems,
};

#[derive(Resource)]
pub struct MovingBoxMeta {
    pub camera: Entity,
    pub spawn: SystemId<In<Timestep>>,
}

impl SimulationMeta for MovingBoxMeta {
    fn get(&self) -> (Entity, SystemId<In<Timestep>>) {
        (self.camera, self.spawn)
    }
}

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

const RENDER_LAYER: usize = 2;

struct Systems;

impl TimesteppedSystems for Systems {
    fn get_systems_for_timestep<T: Component>() -> ScheduleConfigs<ScheduleSystem> {
        // (handle_input, move_boxes::<T>, check_sensors::<T>)
        (move_boxes::<T>, check_sensors::<T>).chain().into_configs()
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems_with_timestep::<Systems>()
        .add_systems(Update, handle_input);
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
    commands.insert_resource(MovingBoxMeta { camera, spawn });

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
    **description = "Moving box controls: 'A', 'Left', 'D', 'Right'".into();

    const BOX_SIZE: f32 = 75.0;
    const MOVING_BOX_SIZE: f32 = 70.0;

    const BASE_Y: f32 = -350.0;

    let y = ((timestep.0 as u8).ilog2() as f32 * BOX_SIZE) + BASE_Y;

    // Sensor boxes
    for i in -2..=2 {
        let i = i as f32;
        let pos = Vec2::new(BOX_SIZE * i * 3.0, y);
        commands.spawn_with_timestep(
            &timestep.0,
            (
                Box(Aabb2d::new(pos, Vec2::splat(BOX_SIZE / 2.0))),
                Transform::from_translation(pos.extend(0.0)),
                RenderLayers::layer(RENDER_LAYER),
                Sprite::from_color(basic::RED, Vec2::splat(BOX_SIZE)),
            ),
        );
    }

    // Moving box
    commands.spawn_with_timestep(
        &timestep.0,
        (
            Movement {
                speed: 1000.0,
                drag: 100.0,
            },
            Box(Aabb2d::new(Vec2::ZERO, Vec2::splat(MOVING_BOX_SIZE / 2.0))),
            Transform::from_translation(Vec3::new(0.0, y, 0.0)),
            RenderLayers::layer(RENDER_LAYER),
            Sprite::from_color(basic::LIME, Vec2::splat(MOVING_BOX_SIZE)),
        ),
    );
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

fn move_boxes<T: Component>(
    mut movers: Query<(&mut Transform, &mut Velocity, &Movement), With<T>>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, configuration) in movers.iter_mut() {
        match *input {
            Input::Left => velocity.0 = -configuration.speed * time.delta_secs(),
            Input::None => velocity
                .0
                .smooth_nudge(&0.0, configuration.drag, time.delta_secs()),
            Input::Right => velocity.0 = configuration.speed * time.delta_secs(),
        }

        let mut new_x = transform.translation.x + velocity.0;
        if new_x.abs() > 600.0 {
            new_x = -new_x.clamp(-600.0, 600.0);
        }
        transform.translation.x = new_x;
    }
}

fn check_sensors<T: Component>(
    movers: Query<(&Transform, &Box), (With<Movement>, With<T>)>,
    mut sensors: Query<(&Box, &mut Sprite), (Without<Movement>, With<T>)>,
) {
    for (sensor_aabb, mut sprite) in sensors.iter_mut() {
        if movers.iter().any(|(transform, aabb)| {
            sensor_aabb
                .0
                .intersects(&aabb.0.translated_by(transform.translation.xy()))
        }) {
            sprite.color = basic::BLUE.into();
        } else {
            sprite.color = basic::RED.into();
        }
    }
}
