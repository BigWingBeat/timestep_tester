use bevy::{
    color::{ColorCurve, palettes::tailwind},
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
    moving_bars::MovingBarsMeta,
    timestep::{Fixed, NoDelta, SemiFixed, VariableDelta},
};

#[derive(Resource, Clone, Copy, Default)]
pub enum ActiveSimulation {
    #[default]
    LorenzAttractor = 1,
    MouseCursor,
    MovingBars,
}

#[derive(TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Timestep {
    NoDelta = 1,
    VariableDelta = 2,
    SemiFixed = 4,
    Fixed = 8,
}

impl Timestep {
    pub fn index(self) -> usize {
        (self as u8).ilog2() as usize
    }

    pub fn palette(self) -> ColorCurve<Oklaba> {
        match self {
            Self::NoDelta => ColorCurve::new([
                tailwind::PURPLE_500.into(),
                tailwind::PURPLE_700.into(),
                tailwind::FUCHSIA_500.into(),
                tailwind::FUCHSIA_700.into(),
                tailwind::PURPLE_500.into(),
            ])
            .unwrap(),
            Self::VariableDelta => ColorCurve::new([
                tailwind::GREEN_500.into(),
                tailwind::GREEN_700.into(),
                tailwind::EMERALD_500.into(),
                tailwind::EMERALD_700.into(),
                tailwind::GREEN_500.into(),
            ])
            .unwrap(),
            Self::SemiFixed => ColorCurve::new([
                tailwind::SKY_500.into(),
                tailwind::SKY_700.into(),
                tailwind::BLUE_500.into(),
                tailwind::BLUE_700.into(),
                tailwind::SKY_500.into(),
            ])
            .unwrap(),
            Self::Fixed => ColorCurve::new([
                tailwind::ROSE_500.into(),
                tailwind::ROSE_700.into(),
                tailwind::RED_500.into(),
                tailwind::RED_700.into(),
                tailwind::ROSE_500.into(),
            ])
            .unwrap(),
        }
    }
}

pub trait CommandsExt {
    fn spawn_with_timestep(
        &mut self,
        timestep: &Timestep,
        bundle: impl Bundle,
    ) -> EntityCommands<'_>;
}

impl CommandsExt for Commands<'_, '_> {
    fn spawn_with_timestep(
        &mut self,
        timestep: &Timestep,
        bundle: impl Bundle,
    ) -> EntityCommands<'_> {
        match timestep {
            Timestep::NoDelta => self.spawn((NoDelta, bundle)),
            Timestep::VariableDelta => self.spawn((VariableDelta, bundle)),
            Timestep::SemiFixed => self.spawn((SemiFixed, bundle)),
            Timestep::Fixed => self.spawn((Fixed, bundle)),
        }
    }
}

pub trait TimesteppedSystems {
    fn get_systems_for_timestep<T: TimestepComponent>() -> ScheduleConfigs<ScheduleSystem>;
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
        self.iter_names()
            .map(|(_, timestep)| timestep.bits().try_into().unwrap())
    }
}

impl From<Timestep> for ActiveTimesteps {
    fn from(timestep: Timestep) -> Self {
        match timestep {
            Timestep::NoDelta => Self::NO_DELTA,
            Timestep::VariableDelta => Self::VARIABLE_DELTA,
            Timestep::SemiFixed => Self::SEMI_FIXED,
            Timestep::Fixed => Self::FIXED,
        }
    }
}

pub trait TimestepComponent: Component {
    const TIMESTEP: Timestep;
}

impl TimestepComponent for NoDelta {
    const TIMESTEP: Timestep = Timestep::NoDelta;
}

impl TimestepComponent for VariableDelta {
    const TIMESTEP: Timestep = Timestep::VariableDelta;
}

impl TimestepComponent for SemiFixed {
    const TIMESTEP: Timestep = Timestep::SemiFixed;
}

impl TimestepComponent for Fixed {
    const TIMESTEP: Timestep = Timestep::Fixed;
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
        .add_systems(PostStartup, setup);
}

fn setup(mut commands: Commands) {
    commands.run_system_cached(respawn);
}

pub fn respawn(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera)>,
    active_simulation: Res<ActiveSimulation>,
    active_timesteps: Res<ActiveTimesteps>,
    despawn_systems: Res<DespawnSystems>,
    lorenz_attractor: Res<LorenzAttractorMeta>,
    mouse_cursor: Res<MouseCursorMeta>,
    moving_bars: Res<MovingBarsMeta>,
) {
    for &system in &despawn_systems.0 {
        commands.run_system(system);
    }

    let (active_camera, spawn) = match *active_simulation {
        ActiveSimulation::LorenzAttractor => lorenz_attractor.get(),
        ActiveSimulation::MouseCursor => mouse_cursor.get(),
        ActiveSimulation::MovingBars => moving_bars.get(),
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
