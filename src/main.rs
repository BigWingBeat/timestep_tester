use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin, feathers::FeathersPlugins, prelude::*,
    window::PresentMode,
};

use crate::{
    simulation::{lorenz_attractor_plugin, mouse_cursor_plugin, moving_bars_plugin},
    timestep::SemiFixed,
};

mod configuration;
mod simulation;
mod timestep;
mod ui;
mod update_cadence;

fn main() -> AppExit {
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
            lorenz_attractor_plugin,
            mouse_cursor_plugin,
            moving_bars_plugin,
            timestep::plugin,
            ui::plugin,
            update_cadence::UpdateCadencePlugin::default().add_schedule(SemiFixed),
        ))
        .run()
}
