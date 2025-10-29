use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin, feathers::FeathersPlugins, prelude::*,
    window::PresentMode,
};
use clap::{Parser, ValueEnum};

mod configuration;
mod lorenz_attractor;
mod mouse_cursor;
mod moving_box;
mod timestep;
mod ui;

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
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Mailbox,
                    ..default()
                }),
                ..default()
            }),
            FeathersPlugins,
            FpsOverlayPlugin::default(),
        ))
        .add_plugins((
            configuration::plugin,
            lorenz_attractor::plugin,
            mouse_cursor::plugin,
            moving_box::plugin,
            timestep::plugin,
            ui::plugin,
        ))
        .run()
}
