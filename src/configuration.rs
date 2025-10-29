use bevy::{
    ecs::{
        schedule::ScheduleConfigs,
        system::{ScheduleSystem, SystemId},
    },
    prelude::*,
};
use bitflags::bitflags;
use num_enum::TryFromPrimitive;

use crate::{
    lorenz_attractor::LorenzAttractorMeta,
    mouse_cursor::MouseCursorMeta,
    moving_box::MovingBoxMeta,
    timestep::{Fixed, NoDelta, SemiFixed, VariableDelta},
};

#[derive(Resource, Clone, Copy, Default)]
enum ActiveSimulation {
    #[default]
    LorenzAttractor = 1,
    MouseCursor,
    MovingBox,
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Timestep {
    NoDelta = 1,
    VariableDelta = 2,
    SemiFixed = 4,
    Fixed = 8,
}

pub trait CommandsExt {
    fn spawn_with_timestep(&mut self, timestep: &Timestep, bundle: impl Bundle) -> EntityCommands;
}

impl CommandsExt for Commands<'_, '_> {
    fn spawn_with_timestep(&mut self, timestep: &Timestep, bundle: impl Bundle) -> EntityCommands {
        match timestep {
            Timestep::NoDelta => self.spawn((NoDelta, bundle)),
            Timestep::VariableDelta => self.spawn((VariableDelta, bundle)),
            Timestep::SemiFixed => self.spawn((SemiFixed, bundle)),
            Timestep::Fixed => self.spawn((Fixed, bundle)),
        }
    }
}

pub trait TimesteppedSystems {
    fn get_systems_for_timestep<T: Component>() -> ScheduleConfigs<ScheduleSystem>;
}

pub trait AppExt {
    fn add_systems_with_timestep<T: TimesteppedSystems>(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn add_systems_with_timestep<T: TimesteppedSystems>(&mut self) -> &mut Self {
        self.add_systems(NoDelta, T::get_systems_for_timestep::<NoDelta>());
        self.add_systems(
            VariableDelta,
            T::get_systems_for_timestep::<VariableDelta>(),
        );
        self.add_systems(SemiFixed, T::get_systems_for_timestep::<SemiFixed>());
        self.add_systems(Fixed, T::get_systems_for_timestep::<Fixed>());
        self
    }
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

#[derive(Resource, Default)]
pub struct DespawnSystems(pub Vec<SystemId>);

pub trait SimulationMeta {
    fn get(&self) -> (Entity, SystemId<In<Timestep>>);
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ActiveSimulation>()
        .init_resource::<ActiveTimesteps>()
        .init_resource::<DespawnSystems>()
        .add_systems(PostStartup, setup)
        .add_systems(Update, handle_input);
}

fn setup(mut commands: Commands) {
    commands.run_system_cached(respawn);
}

fn respawn(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera)>,
    active_simulation: Res<ActiveSimulation>,
    active_timesteps: Res<ActiveTimesteps>,
    despawn_systems: Res<DespawnSystems>,
    lorenz_attractor: Res<LorenzAttractorMeta>,
    mouse_cursor: Res<MouseCursorMeta>,
    moving_box: Res<MovingBoxMeta>,
) {
    for &system in &despawn_systems.0 {
        commands.run_system(system);
    }

    let (active_camera, spawn) = match *active_simulation {
        ActiveSimulation::LorenzAttractor => lorenz_attractor.get(),
        ActiveSimulation::MouseCursor => mouse_cursor.get(),
        ActiveSimulation::MovingBox => moving_box.get(),
    };

    for (entity, mut camera) in cameras.iter_mut() {
        if active_camera == entity {
            camera.is_active = true;
            commands.entity(entity).insert(IsDefaultUiCamera);
        } else {
            camera.is_active = false;
            commands.entity(entity).remove::<IsDefaultUiCamera>();
        }
    }

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
