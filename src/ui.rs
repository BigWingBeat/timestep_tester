use bevy::{
    feathers::{self, dark_theme::create_dark_theme, theme::UiTheme},
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    window::PresentMode,
};

mod frame_pacing;
mod simulation;
mod timesteps;

pub use simulation::SimulationDescription;

use crate::ui::{frame_pacing::presentation_modes, simulation::simulation, timesteps::timesteps};

const GAP_SIZE: Val = Val::Px(12.0);

#[derive(Component)]
struct WindowPresentMode(PresentMode);

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
            padding: UiRect::all(GAP_SIZE),
            border: UiRect::all(Val::Px(2.0)),
            flex_direction: FlexDirection::Column,
            row_gap: GAP_SIZE,
            ..default()
        },
        BackgroundColor(feathers::palette::GRAY_1),
        BorderColor::all(feathers::palette::WARM_GRAY_1),
        BorderRadius::all(Val::Px(4.0)),
        TabGroup::default(),
        children![simulation(), timesteps(), presentation_modes(),],
    ));
}
