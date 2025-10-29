use bevy::{
    feathers::{
        self,
        controls::{checkbox, radio},
        dark_theme::create_dark_theme,
        theme::UiTheme,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange},
};

use crate::configuration::{ActiveSimulation, ActiveTimesteps, respawn};

#[derive(Component)]
pub struct SimulationDescription;

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

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
            TabGroup::default(),
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
                .observe(
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
                )
                .with_children(|radios| {
                    radios.spawn(Text::new("Switch active simulation:"));

                    radios
                        .spawn(radio(
                            (
                                Checked,
                                SimulationRadioButton(ActiveSimulation::LorenzAttractor),
                            ),
                            (),
                        ))
                        .with_child(Text::new("Lorenz Attractor"));

                    radios
                        .spawn(radio(
                            SimulationRadioButton(ActiveSimulation::MouseCursor),
                            (),
                        ))
                        .with_child(Text::new("Mouse Cursor"));

                    radios
                        .spawn(radio(
                            SimulationRadioButton(ActiveSimulation::MovingBox),
                            (),
                        ))
                        .with_child(Text::new("Moving Box"));
                });

            children
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },))
                .with_children(
                    |toggles: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, ChildOf>| {
                        toggles.spawn(Text::new("Timestep toggles:"));

                        toggles
                            .spawn(checkbox((), ()))
                            .with_child(Text::new("No Delta Time"))
                            .observe(
                                |on: On<ValueChange<bool>>,
                                 mut active_timesteps: ResMut<ActiveTimesteps>,
                                 mut commands: Commands| {
                                    active_timesteps.toggle(ActiveTimesteps::NO_DELTA);
                                    if on.value {
                                        commands.entity(on.source).insert(Checked);
                                    } else {
                                        commands.entity(on.source).remove::<Checked>();
                                    }
                                    commands.run_system_cached(respawn);
                                },
                            );

                        toggles
                            .spawn(checkbox((), ()))
                            .with_child(Text::new("Variable Delta Time"))
                            .observe(
                                |on: On<ValueChange<bool>>,
                                 mut active_timesteps: ResMut<ActiveTimesteps>,
                                 mut commands: Commands| {
                                    active_timesteps.toggle(ActiveTimesteps::VARIABLE_DELTA);
                                    if on.value {
                                        commands.entity(on.source).insert(Checked);
                                    } else {
                                        commands.entity(on.source).remove::<Checked>();
                                    }
                                    commands.run_system_cached(respawn);
                                },
                            );

                        toggles
                            .spawn(checkbox(Checked, ()))
                            .with_child(Text::new("Semi-Fixed Timestep"))
                            .observe(
                                |on: On<ValueChange<bool>>,
                                 mut active_timesteps: ResMut<ActiveTimesteps>,
                                 mut commands: Commands| {
                                    active_timesteps.toggle(ActiveTimesteps::SEMI_FIXED);
                                    if on.value {
                                        commands.entity(on.source).insert(Checked);
                                    } else {
                                        commands.entity(on.source).remove::<Checked>();
                                    }
                                    commands.run_system_cached(respawn);
                                },
                            );

                        toggles
                            .spawn(checkbox((), ()))
                            .with_child(Text::new("Fixed Timestep"))
                            .observe(
                                |on: On<ValueChange<bool>>,
                                 mut active_timesteps: ResMut<ActiveTimesteps>,
                                 mut commands: Commands| {
                                    active_timesteps.toggle(ActiveTimesteps::FIXED);
                                    if on.value {
                                        commands.entity(on.source).insert(Checked);
                                    } else {
                                        commands.entity(on.source).remove::<Checked>();
                                    }
                                    commands.run_system_cached(respawn);
                                },
                            );
                    },
                );

            children.spawn((Text::default(), SimulationDescription));
        });
}
