use bevy::{
    feathers::{
        self,
        controls::{checkbox, radio},
        dark_theme::create_dark_theme,
        theme::UiTheme,
    },
    prelude::*,
    ui::Checked,
    ui_widgets::RadioGroup,
};

#[derive(Component)]
pub struct SimulationDescription;

pub fn plugin(app: &mut App) {
    app.insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ClearColor(feathers::palette::GRAY_0))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                top: Val::Px(120.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(feathers::palette::GRAY_1),
            BorderColor::all(feathers::palette::WARM_GRAY_1),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|children| {
            children
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    RadioGroup,
                ))
                .with_children(|radios| {
                    radios.spawn(Text::new("Switch active simulation:"));

                    radios
                        .spawn(radio(Checked, ()))
                        .with_child(Text::new("'1': Lorenz Attractor"));

                    radios
                        .spawn(radio((), ()))
                        .with_child(Text::new("'2': Mouse Cursor"));

                    radios
                        .spawn(radio((), ()))
                        .with_child(Text::new("'3': Moving Box"));
                });

            children
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },))
                .with_children(|toggles| {
                    toggles.spawn(Text::new("Timestep toggles:"));

                    toggles
                        .spawn(checkbox((), ()))
                        .with_child(Text::new("'4': No Delta Time"));

                    toggles
                        .spawn(checkbox((), ()))
                        .with_child(Text::new("'5': Variable Delta Time"));

                    toggles
                        .spawn(checkbox(Checked, ()))
                        .with_child(Text::new("'6': Semi-Fixed Timestep"));

                    toggles
                        .spawn(checkbox((), ()))
                        .with_child(Text::new("'7': Fixed Timestep"));
                });

            children.spawn((Text::default(), SimulationDescription));
        });
}
