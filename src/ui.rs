use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::{
        self,
        controls::{checkbox, radio},
        dark_theme::create_dark_theme,
        theme::UiTheme,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange, observe},
};

use crate::configuration::{ActiveSimulation, ActiveTimesteps, respawn};

#[derive(Component)]
pub struct SimulationDescription;

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

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

pub fn plugin(app: &mut App) {
    app.insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ClearColor(feathers::palette::GRAY_0))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn((
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
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
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
                    }
                ),
                children![
                    Text::new("Switch active simulation:"),
                    (radio(
                        (
                            Checked,
                            SimulationRadioButton(ActiveSimulation::LorenzAttractor)
                        ),
                        Spawn(Text::new("Lorenz Attractor"))
                    )),
                    (radio(
                        SimulationRadioButton(ActiveSimulation::MouseCursor),
                        Spawn(Text::new("Mouse Cursor"))
                    )),
                    (radio(
                        SimulationRadioButton(ActiveSimulation::MovingBox),
                        Spawn(Text::new("Moving Box"))
                    ))
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    Text::new("Timestep Toggles:"),
                    checkbox(
                        observe(toggle_timestep(ActiveTimesteps::NO_DELTA)),
                        Spawn(Text::new("No Delta Time"))
                    ),
                    checkbox(
                        observe(toggle_timestep(ActiveTimesteps::VARIABLE_DELTA)),
                        Spawn(Text::new("Variable Delta Time"))
                    ),
                    checkbox(
                        (
                            Checked,
                            observe(toggle_timestep(ActiveTimesteps::SEMI_FIXED)),
                        ),
                        Spawn(Text::new("Semi-Fixed Timestep"))
                    ),
                    checkbox(
                        observe(toggle_timestep(ActiveTimesteps::FIXED)),
                        Spawn(Text::new("Fixed Timestep"))
                    ),
                ]
            ),
            (Text::default(), SimulationDescription)
        ],
    ));
}
