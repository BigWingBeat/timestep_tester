use std::time::{Duration, Instant};

use bevy::{
    feathers::controls::{SliderProps, slider},
    prelude::*,
    ui_widgets::{SliderPrecision, SliderValue, ValueChange, observe},
};

use crate::ui::describe;

#[derive(Resource, Default)]
struct LagConfig {
    frames_delay: u32,
    lag_duration_ms: u64,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LagConfig>()
        .add_systems(Update, lag_system);
}

pub fn lag() -> impl Bundle {
    children![
        describe(
            Text::new("Artificial Lag"),
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
                max: 1000.0
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
                max: 1000.0
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
    ]
}

fn lag_system(config: Res<LagConfig>, mut counter: Local<u32>) {
    if *counter >= config.frames_delay {
        *counter = 0;

        let duration = Duration::from_millis(config.lag_duration_ms);
        let now = Instant::now();
        while now.elapsed() < duration {
            std::hint::spin_loop();
        }
    } else {
        *counter += 1;
    }
}
