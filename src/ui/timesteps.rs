use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::checkbox,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
};

use crate::configuration::{ActiveTimesteps, respawn};

fn toggle_timestep(timestep: ActiveTimesteps) -> impl ObserverSystem<ValueChange<bool>, ()> {
    IntoObserverSystem::into_system(
        move |on: On<ValueChange<bool>>,
              mut active_timesteps: ResMut<ActiveTimesteps>,
              mut commands: Commands| {
            active_timesteps.set(timestep, on.value);
            if on.value {
                commands.entity(on.source).insert(Checked);
            } else {
                commands.entity(on.source).remove::<Checked>();
            }
            commands.run_system_cached(respawn);
        },
    )
}

pub fn timesteps() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            Text::new("Timestep Toggles:"),
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::NO_DELTA)),
                Spawn(Text::new("No Delta Time"))
            ),
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::VARIABLE_DELTA)),
                Spawn(Text::new("Variable Delta Time"))
            ),
            checkbox(
                (
                    Checked,
                    observe(toggle_timestep(ActiveTimesteps::SEMI_FIXED)),
                ),
                Spawn(Text::new("Semi-Fixed Timestep"))
            ),
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::FIXED)),
                Spawn(Text::new("Fixed Timestep"))
            ),
        ],
    )
}
