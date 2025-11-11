use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin, feathers::FeathersPlugins, prelude::*,
    window::PresentMode,
};

use crate::timestep::SemiFixed;

mod configuration;
mod lorenz_attractor;
mod mouse_cursor;
mod moving_bars;
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
            lorenz_attractor::plugin,
            mouse_cursor::plugin,
            moving_bars::plugin,
            timestep::plugin,
            ui::plugin,
            update_cadence::UpdateCadencePlugin::default().add_schedule(SemiFixed),
        ))
        .run()
}
