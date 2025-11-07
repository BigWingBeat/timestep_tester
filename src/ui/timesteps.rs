use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::checkbox,
    prelude::*,
    ui::Checked,
    ui_widgets::{ValueChange, observe},
};

use crate::{
    configuration::{ActiveTimesteps, respawn},
    ui::describe,
};

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
    children![
        Text::new("Timestep Toggles:"),
        describe(
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::NO_DELTA)),
                Spawn(Text::new("No Delta Time"))
            ),
            "Updates every frame, without utilising delta time. Simulation speed is directly tied to framerate."
        ),
        describe(
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::VARIABLE_DELTA)),
                Spawn(Text::new("Variable Delta Time"))
            ),
            "Updates every frame, utilising delta time. Non-deterministic, destabilizes simulation at extremely low framerates."
        ),
        describe(
            checkbox(
                (
                    Checked,
                    observe(toggle_timestep(ActiveTimesteps::SEMI_FIXED)),
                ),
                Spawn(Text::new("Semi-Fixed Timestep"))
            ),
            "Updates one or more times per frame, utilising a capped delta time. Non-deterministic, breaks down with extremely slow simulation updates."
        ),
        describe(
            checkbox(
                observe(toggle_timestep(ActiveTimesteps::FIXED)),
                Spawn(Text::new("Fixed Timestep"))
            ),
            "Updates zero or more times per frame, utilising a fixed delta time. Can be deterministic, causes visual issues, breaks down with extremely slow simulation updates."
        )
    ]
}
