use bevy::{
    feathers::controls::{SliderProps, slider},
    prelude::*,
};

use crate::ui::describe;

pub fn update_rate(root: impl Bundle) -> impl Bundle {
    (
        root,
        children![
            Text::new("Simulation Rate:"),
            describe(
                slider(
                    SliderProps {
                        value: 64.0,
                        min: 1.0,
                        max: 1000.0
                    },
                    (),
                ),
                "The target frequency in Hz at which the simulation tries to run, independant of the framerate."
            ),
            describe(
                Text::new("Frame Pacing:"),
                "Settings to control the framerate of the whole application."
            ),
        ],
    )
}
