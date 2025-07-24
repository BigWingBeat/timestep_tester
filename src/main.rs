use bevy::prelude::*;
use clap::{Parser, ValueEnum};

mod lorenz_attractor;
mod moving_box;

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
        .add_plugins((lorenz_attractor::plugin, moving_box::plugin))
        .run()
}
