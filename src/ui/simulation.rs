use bevy::{
    feathers::controls::radio,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange, observe},
};

use crate::{
    configuration::{ActiveSimulation, respawn},
    ui::GAP_SIZE,
};

#[derive(Component)]
pub struct SimulationDescription;

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

pub fn simulation() -> impl Bundle {
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
            },
        ),
        children![
            Text::new("Switch Active Simulation:"),
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
            )),
            (
                // Add a gap between the description and the radio buttons
                Node {
                    margin: UiRect::top(GAP_SIZE),
                    ..default()
                },
                Text::default(),
                SimulationDescription
            ),
        ],
    )
}
