use std::time::Duration;

use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::{SliderProps, checkbox, slider},
    prelude::*,
    ui::Checked,
    ui_widgets::{SliderValue, ValueChange, observe},
};

use crate::{
    configuration::{ActiveTimesteps, respawn},
    timestep::SimulationDelta,
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
        describe(
            Text::new("Simulation Rate:"),
            "The target frequency in Hz at which the simulation tries to run, independant of the framerate."
        ),
        slider(
            SliderProps {
                value: 64.0,
                min: 1.0,
                max: 1000.0
            },
            observe(
                |on: On<ValueChange<f32>>,
                 mut commands: Commands,
                 mut simulation_delta: ResMut<SimulationDelta>| {
                    commands.entity(on.source).insert(SliderValue(on.value));
                    simulation_delta.0 = Duration::from_secs_f32(on.value.recip());
                }
            ),
        ),
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
