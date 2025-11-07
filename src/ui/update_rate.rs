use bevy::{
    feathers::controls::{SliderProps, slider},
    prelude::*,
};

use crate::ui::describe;

pub fn update_rate() -> impl Bundle {
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
            (),
        ),
        describe(
            Text::new("Frame Pacing:"),
            "Settings to control the framerate of the whole application."
        ),
    ]
}
