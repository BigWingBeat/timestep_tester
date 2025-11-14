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

use crate::ui::{
    presentation_modes::presentation_modes,
    simulation::simulation,
    tabs::{TabCorners, tabs},
    timesteps::timesteps,
    update_rate::update_rate,
};

const GAP_SIZE: Val = Val::Px(12.0);
const MAX_WIDTH: Val = Val::Px(720.0);

/// Largest range sliders can have without skipping over some integers.
/// Equal to `(MAX_WIDTH / 2.0) + (GAP_SIZE * 2.0)`
const SLIDER_PRECISION: f32 = 336.0;

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

#[derive(Component, Default)]
struct TopLevelTabs;

pub fn plugin(app: &mut App) {
    app.add_plugins((update_rate::plugin, simulation::plugin))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ClearColor(feathers::palette::GRAY_0))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    let (buttons, contents) = tabs![
        TopLevelTabs,
        TabCorners::Top,
        ("Simulation", simulation()),
        ("Timesteps", timesteps()),
        ("Presentation Modes", presentation_modes()),
        ("Update Rate", update_rate()),
    ];

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
        children![
            buttons,
            (
                Node {
                    padding: UiRect::all(GAP_SIZE),
                    border: UiRect::all(Val::Px(2.0)).with_top(Val::ZERO),
                    max_width: MAX_WIDTH,
                    ..default()
                },
                BackgroundColor(feathers::palette::GRAY_1),
                BorderColor::all(feathers::palette::WARM_GRAY_1),
                BorderRadius::all(Val::Px(4.0)).with_top(Val::ZERO),
                Children::spawn(contents),
            )
        ],
    ));
}
