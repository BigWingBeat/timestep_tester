use bevy::{
    feathers::{self, dark_theme::create_dark_theme, theme::UiTheme},
    input_focus::tab_navigation::TabGroup,
    prelude::*,
};

mod presentation_modes;
mod simulation;
mod tabs;
mod timesteps;
mod update_rate;

pub use simulation::SimulationDescription;

use crate::ui::{
    presentation_modes::presentation_modes, simulation::simulation, tabs::tabs,
    timesteps::timesteps, update_rate::update_rate,
};

const GAP_SIZE: Val = Val::Px(12.0);
const MAX_WIDTH: Val = Val::Px(720.0);

/// Add a description below a node
fn describe(node: impl Bundle, description: impl Into<String>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            node,
            (Text::new(description), TextFont::from_font_size(18.0)),
        ],
    )
}

pub fn plugin(app: &mut App) {
    app.insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ClearColor(feathers::palette::GRAY_0))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: GAP_SIZE,
            top: GAP_SIZE * 10.0,
            max_width: MAX_WIDTH,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        TabGroup::default(),
        tabs![
            ("Simulation", simulation()),
            ("Timesteps", timesteps()),
            ("Presentation Modes", presentation_modes()),
            ("Update Rate", update_rate()),
        ],
    ));
}
