use bevy::{
    feathers::controls::radio,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange, observe},
};

use crate::{
    configuration::{ActiveSimulation, respawn},
    ui::describe,
};

#[derive(Component)]
pub struct SimulationDescription;

#[derive(Component)]
struct SimulationRadioButton(ActiveSimulation);

pub fn simulation(root: impl Bundle) -> impl Bundle {
    (
        root,
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
            describe(
                radio(
                    (
                        Checked,
                        SimulationRadioButton(ActiveSimulation::LorenzAttractor)
                    ),
                    Spawn(Text::new("Lorenz Attractor"))
                ),
                "A chaotic system that provides an exaggerated visualisation of non-determinism."
            ),
            describe(
                radio(
                    SimulationRadioButton(ActiveSimulation::MouseCursor),
                    Spawn(Text::new("Mouse Cursor"))
                ),
                "Boxes that follow the mouse, useful for visualising latency."
            ),
            describe(
                radio(
                    SimulationRadioButton(ActiveSimulation::MovingBox),
                    Spawn(Text::new("Moving Box"))
                ),
                "Fast-moving, high-contrast images, useful for visualising screen tearing and stuttering."
            ),
            (Text::default(), SimulationDescription),
        ],
    )
}
