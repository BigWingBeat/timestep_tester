use std::time::Duration;

use bevy::{
    feathers::controls::{SliderProps, radio, slider},
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, SliderPrecision, SliderValue, ValueChange, observe},
};

use crate::{
    configuration::{ActiveSimulation, respawn},
    ui::{SLIDER_PRECISION, describe},
};

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

#[derive(Resource, Default)]
struct LagConfig {
    frames_delay: u32,
    lag_duration_ms: u64,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LagConfig>()
        .add_systems(Update, lag_system);
}

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
            describe(
                Text::new("Artificial Lag:"),
                "Fake a heavier computational load to manually slow the app down."
            ),
            describe(
                Text::new("Lag Frequency"),
                "How many frames to wait between each lag frame."
            ),
            slider(
                SliderProps {
                    value: 0.0,
                    min: 0.0,
                    max: SLIDER_PRECISION
                },
                (
                    SliderPrecision(0),
                    observe(
                        |on: On<ValueChange<f32>>,
                         mut commands: Commands,
                         mut config: ResMut<LagConfig>| {
                            commands.entity(on.source).insert(SliderValue(on.value));
                            config.frames_delay = on.value as u32;
                        }
                    )
                ),
            ),
            describe(
                Text::new("Lag Duration"),
                "Extra time in milliseconds to wait for, on each lag frame."
            ),
            slider(
                SliderProps {
                    value: 0.0,
                    min: 0.0,
                    max: SLIDER_PRECISION
                },
                (
                    SliderPrecision(0),
                    observe(
                        |on: On<ValueChange<f32>>,
                         mut commands: Commands,
                         mut config: ResMut<LagConfig>| {
                            commands.entity(on.source).insert(SliderValue(on.value));
                            config.lag_duration_ms = on.value as u64;
                        }
                    )
                ),
            ),
        ],
    )
}

fn lag_system(config: Res<LagConfig>, mut counter: Local<u32>) {
    if *counter >= config.frames_delay {
        *counter = 0;

        let duration = Duration::from_millis(config.lag_duration_ms);
        spin_sleep::sleep(duration);
    } else {
        *counter += 1;
    }
}
