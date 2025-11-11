use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::RenderLayers,
    color::{ColorCurve, palettes::tailwind},
    core_pipeline::tonemapping::Tonemapping,
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    image::TextureFormatPixelInfo,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use bitflags::Flags;

use crate::{
    configuration::{
        ActiveTimesteps, AppExt, CommandsExt, DespawnSystems, SimulationMeta, Timestep,
        TimestepComponent, TimesteppedSystems,
    },
    ui::SimulationDescription,
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

#[derive(Resource)]
struct Textures([Handle<Image>; ActiveTimesteps::FLAGS.len()]);

fn texture(timestep: Timestep) -> Image {
    // match timestep {
    //     Timestep::NoDelta => todo!(),
    //     Timestep::VariableDelta => todo!(),
    //     Timestep::SemiFixed => todo!(),
    //     Timestep::Fixed => todo!(),
    // }

    let format = TextureFormat::bevy_default();
    // If the default format changes the following code would need to change as well
    assert_eq!(format, TextureFormat::Rgba8UnormSrgb);
    assert!(matches!(format.pixel_size(), Ok(4)));

    let colours = ColorCurve::<Oklcha>::new([
        tailwind::SKY_500.into(),
        tailwind::SKY_800.into(),
        tailwind::BLUE_800.into(),
    ])
    .unwrap();

    const RESOLUTION: u8 = 64;

    let data = colours
        .domain()
        .spaced_points(RESOLUTION.into())
        .unwrap()
        .flat_map(|t| {
            // Adapted from `Image::set_color_at_internal`
            Srgba::from(colours.sample_unchecked(t))
                .to_f32_array()
                .map(|b| (b * u8::MAX as f32) as u8)
        })
        .collect();

    Image::new(
        Extent3d {
            width: RESOLUTION.into(),
            ..default()
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::default(),
    )
}

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

fn setup(
    mut commands: Commands,
    mut despawns: ResMut<DespawnSystems>,
    mut images: ResMut<Assets<Image>>,
) {
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

    let textures = [
        images.add(texture(Timestep::NoDelta)),
        images.add(texture(Timestep::VariableDelta)),
        images.add(texture(Timestep::SemiFixed)),
        images.add(texture(Timestep::Fixed)),
    ];
    commands.insert_resource(Textures(textures));
}

fn despawn(mut commands: Commands, bars: Query<Entity, With<Bar>>) {
    for entity in bars.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn(
    timestep: In<Timestep>,
    mut commands: Commands,
    mut description: Single<&mut Text, With<SimulationDescription>>,
    textures: Res<Textures>,
) {
    **description = "Moving Bars".into();

    const WIDTH: f32 = 80.0;

    let count = (MAX_X / (WIDTH * 2.0)) as u8;
    for i in 0..count {
        // The resize_y system handles setting y pos and height, so we don't bother duplicating that work here
        let x = (i as f32) * WIDTH * 2.0;
        commands.spawn_with_timestep(
            &timestep.0,
            (
                Transform::from_xyz(x, 0.0, 1.0),
                RenderLayers::layer(RENDER_LAYER),
                Sprite {
                    image: textures.0[timestep.0.index()].clone(),
                    custom_size: Some(Vec2::splat(WIDTH)),
                    ..default()
                },
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
    let y = ((above_count as f32) * height) - offset;

    for (mut transform, mut sprite) in bars.iter_mut() {
        let new_x = transform.translation.x + (MOVE_SPEED * time.delta_secs());
        transform.translation.x = new_x % MAX_X;
        transform.translation.y = y;
        sprite.custom_size.as_mut().unwrap().y = height;
    }
}
