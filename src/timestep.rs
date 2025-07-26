use bevy::{ecs::system::SystemId, prelude::*};
use bitflags::{Flags, bitflags};
use num_enum::TryFromPrimitive;

use crate::{
    lorenz_attractor::SpawnLorenzAttractor, mouse_cursor::SpawnMouseCursor,
    moving_box::SpawnMovingBox,
};

#[derive(Resource, Clone, Copy, Default)]
enum ActiveSimulation {
    #[default]
    LorenzAttractor = 1,
    MouseCursor,
    MovingBox,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Timestep {
    NoDelta = 1,
    VariableDelta = 2,
    SemiFixed = 4,
    Fixed = 8,
}

bitflags! {
    #[derive(Resource, Clone, Copy)]
    pub struct ActiveTimesteps: u8 {
        const NO_DELTA = 1;
        const VARIABLE_DELTA = 2;
        const SEMI_FIXED = 4;
        const FIXED = 8;
    }
}

impl Default for ActiveTimesteps {
    fn default() -> Self {
        Self::SEMI_FIXED
    }
}

impl ActiveTimesteps {
    fn iter_timesteps(&self) -> impl Iterator<Item = Timestep> {
        self.iter()
            .map(|timestep| timestep.bits().try_into().unwrap())
    }
}

#[derive(Component)]
struct ActiveSimulationDescription;

#[derive(Component)]
struct ActiveTimestepsDescription;

#[derive(Component)]
pub struct SimulationDescription;

#[derive(Resource, Default)]
pub struct DespawnSystems(pub Vec<SystemId>);

pub fn plugin(app: &mut App) {
    app.init_resource::<ActiveSimulation>()
        .init_resource::<ActiveTimesteps>()
        .init_resource::<DespawnSystems>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input);
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|children| {
            children.spawn((TextSpan::default(), ActiveSimulationDescription));
            children.spawn((TextSpan::default(), ActiveTimestepsDescription));
            children.spawn((TextSpan::default(), SimulationDescription));
        });

    commands.run_system_cached(respawn);
}

fn respawn(
    mut commands: Commands,
    mut active_simulation_description: Single<
        &mut TextSpan,
        (
            With<ActiveSimulationDescription>,
            Without<ActiveTimestepsDescription>,
        ),
    >,
    mut active_timesteps_description: Single<
        &mut TextSpan,
        (
            With<ActiveTimestepsDescription>,
            Without<ActiveSimulationDescription>,
        ),
    >,
    active_simulation: Res<ActiveSimulation>,
    active_timesteps: Res<ActiveTimesteps>,
    despawn_systems: Res<DespawnSystems>,
    lorenz_attractor: Res<SpawnLorenzAttractor>,
    mouse_cursor: Res<SpawnMouseCursor>,
    moving_box: Res<SpawnMovingBox>,
) {
    let mut active_sim_text = vec![
        "Switch active simulation:\n'1': Lorenz Attractor",
        "\n'2': Mouse Cursor",
        "\n'3': Moving Box",
    ];

    active_sim_text.insert(*active_simulation as _, " (*)");
    ***active_simulation_description = active_sim_text.into_iter().collect();

    let mut active_timesteps_text = vec![
        "\n\nTimestep toggles:",
        "\n'4': No Delta Time",
        "\n'5': Variable Delta Time",
        "\n'6': Semi-Fixed Timestep",
        "\n'7': Fixed Timestep",
    ];

    // Reverse order so the inserts don't change the correct indexes for later inserts
    for timestep in ActiveTimesteps::FLAGS
        .iter()
        .filter(|flag| active_timesteps.contains(*flag.value()))
        .map(|timestep| (timestep.value().bits().ilog2() + 2) as _)
        .rev()
    {
        active_timesteps_text.insert(timestep, " (*)");
    }

    ***active_timesteps_description = active_timesteps_text.into_iter().collect();

    for &system in &despawn_systems.0 {
        commands.run_system(system);
    }

    let spawn = match *active_simulation {
        ActiveSimulation::LorenzAttractor => lorenz_attractor.0,
        ActiveSimulation::MouseCursor => mouse_cursor.0,
        ActiveSimulation::MovingBox => moving_box.0,
    };

    for timestep in active_timesteps.iter_timesteps() {
        commands.run_system_with(spawn, timestep);
    }
}

fn handle_input(
    mut commands: Commands,
    mut active_simulation: ResMut<ActiveSimulation>,
    mut active_timesteps: ResMut<ActiveTimesteps>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut any_change = true;

    let lorenz_attractor = keys.just_pressed(KeyCode::Digit1);
    let mouse_cursor = keys.just_pressed(KeyCode::Digit2);
    let moving_box = keys.just_pressed(KeyCode::Digit3);
    match (lorenz_attractor, mouse_cursor, moving_box) {
        (true, false, false) => *active_simulation = ActiveSimulation::LorenzAttractor,
        (false, true, false) => *active_simulation = ActiveSimulation::MouseCursor,
        (false, false, true) => *active_simulation = ActiveSimulation::MovingBox,
        _ => any_change = false,
    }

    let no_delta = keys.just_pressed(KeyCode::Digit4);
    let variable_delta = keys.just_pressed(KeyCode::Digit5);
    let semi_fixed = keys.just_pressed(KeyCode::Digit6);
    let fixed = keys.just_pressed(KeyCode::Digit7);

    if no_delta {
        active_timesteps.toggle(ActiveTimesteps::NO_DELTA);
        any_change = true;
    }

    if variable_delta {
        active_timesteps.toggle(ActiveTimesteps::VARIABLE_DELTA);
        any_change = true;
    }

    if semi_fixed {
        active_timesteps.toggle(ActiveTimesteps::SEMI_FIXED);
        any_change = true;
    }

    if fixed {
        active_timesteps.toggle(ActiveTimesteps::FIXED);
        any_change = true;
    }

    if any_change {
        commands.run_system_cached(respawn);
    }
}
