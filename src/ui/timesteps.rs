use std::time::Duration;

use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::{SliderProps, checkbox, radio, slider},
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, SliderPrecision, SliderValue, ValueChange, observe},
};

use crate::{
    configuration::{ActiveTimesteps, respawn},
    interpolation::InterpolationMode,
    timestep::SimulationDelta,
    ui::{SLIDER_PRECISION, describe},
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
    (
        RadioGroup,
        observe(
            |on: On<ValueChange<Entity>>,
             radios: Query<(Entity, &InterpolationMode)>,
             mut mode: ResMut<InterpolationMode>,
             mut commands: Commands| {
                for (entity, &new_mode) in radios.iter() {
                    if entity == on.value {
                        commands.entity(entity).insert(Checked);
                        *mode = new_mode;
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            },
        ),
        children![
            describe(
                Text::new("Simulation Rate:"),
                "The reciprocal of the fixed delta time value used by the different simulation modes, measured in Hz (updates per second)."
            ),
            slider(
                SliderProps {
                    value: 64.0,
                    min: 1.0,
                    max: SLIDER_PRECISION
                },
                (
                    SliderPrecision(0),
                    observe(
                        |on: On<ValueChange<f32>>,
                         mut commands: Commands,
                         mut simulation_delta: ResMut<SimulationDelta>| {
                            commands.entity(on.source).insert(SliderValue(on.value));
                            simulation_delta.0 = Duration::from_secs_f32(on.value.recip());
                        }
                    )
                ),
            ),
            Text::new("Timestep Toggles:"),
            describe(
                checkbox(
                    observe(toggle_timestep(ActiveTimesteps::NO_DELTA)),
                    Spawn(Text::new("No Delta Time"))
                ),
                "Updates once every render frame, with a fixed delta time value. Simulation speed is proportional to framerate."
            ),
            describe(
                checkbox(
                    observe(toggle_timestep(ActiveTimesteps::VARIABLE_DELTA)),
                    Spawn(Text::new("Variable Delta Time"))
                ),
                "Updates once every render frame, with a dynamic delta time value. Unaffected by the configured Simulation Rate."
            ),
            describe(
                checkbox(
                    (
                        Checked,
                        observe(toggle_timestep(ActiveTimesteps::SEMI_FIXED)),
                    ),
                    Spawn(Text::new("Semi-Fixed Timestep"))
                ),
                "Updates one or more times per frame, with a dynamic delta time value, that is capped by the configured Simulation Rate."
            ),
            describe(
                checkbox(
                    observe(toggle_timestep(ActiveTimesteps::FIXED)),
                    Spawn(Text::new("Fixed Timestep"))
                ),
                "Updates zero or more times per frame, with a fixed delta time value. Causes visual stuttering."
            ),
            describe(
                Text::new("Switch Interpolation Mode:"),
                "Affects how the Fixed timestep is rendered. No effect on other timesteps, or on the simulation."
            ),
            describe(
                radio(InterpolationMode::None, Spawn(Text::new("None"))),
                "No interpolation. Causes visual stuttering."
            ),
            describe(
                radio(
                    (Checked, InterpolationMode::Interpolate),
                    Spawn(Text::new("Interpolate"))
                ),
                "Interpolate between past values. Visual state will lag behind simulation state."
            ),
            describe(
                radio(
                    InterpolationMode::Extrapolate,
                    Spawn(Text::new("Extrapolate"))
                ),
                "Extrapolate to a future value. Visual state will sometimes be temporarily incorrect."
            ),
        ],
    )
}
