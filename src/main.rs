use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use clap::{Parser, ValueEnum};

mod lorenz_attractor;

#[derive(Clone, Copy, ValueEnum)]
enum Mode {
    Variable,
    Fixed,
}

#[derive(Parser)]
struct Args {
    mode: Mode,
}

fn main() -> AppExit {
    let args = Args::parse();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(lorenz_attractor::plugin)
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(-100.0, 150.0, 150.0).looking_at(Vec3::new(0.0, 30.0, 0.0), Vec3::Y),
    ));
}
