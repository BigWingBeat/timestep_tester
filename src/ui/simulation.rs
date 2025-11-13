use std::time::Duration;

use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::{SliderProps, checkbox, radio, slider},
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, SliderValue, ValueChange, observe},
};

use crate::{
    configuration::{ActiveSimulation, ActiveTimesteps, respawn},
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

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

pub fn simulation() -> impl Bundle {
    (
        RadioGroup,
        observe(
            |on: On<ValueChange<Entity>>,
             radios: Query<(Entity, &SimulationRadioButton)>,
             mut active_simulation: ResMut<ActiveSimulation>,
             mut commands: Commands| {
                for (entity, simulation) in radios.iter() {
                    if entity == on.value {
                        commands.entity(entity).insert(Checked);
                        *active_simulation = simulation.0;
                        commands.run_system_cached(respawn);
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
            Text::new("Switch Active Simulation:"),
            describe(
                radio(
                    (
                        Checked,
                        SimulationRadioButton(ActiveSimulation::LorenzAttractor)
                    ),
                    Spawn(Text::new("Lorenz Attractor"))
                ),
                "A chaotic system that provides an exaggerated visualisation of non-determinism."
            ),
            describe(
                radio(
                    SimulationRadioButton(ActiveSimulation::MouseCursor),
                    Spawn(Text::new("Mouse Cursor"))
                ),
                "Boxes that follow the mouse, useful for visualising latency."
            ),
            describe(
                radio(
                    SimulationRadioButton(ActiveSimulation::MovingBars),
                    Spawn(Text::new("Moving Bars"))
                ),
                "High-contrast vertical bars, useful for visualising screen tearing and stuttering."
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
            )
        ],
    )
}
