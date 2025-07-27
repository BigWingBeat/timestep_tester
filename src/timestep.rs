use std::time::Duration;

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

#[derive(Resource)]
pub struct SimulationDelta(Duration);

#[derive(Component, ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct NoDelta;

#[derive(Component, ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct VariableDelta;

#[derive(Component, ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct SemiFixed;

#[derive(Component, ScheduleLabel, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Fixed;

pub fn plugin(app: &mut App) {
    app.insert_resource(SimulationDelta(Duration::from_secs_f32(1.0 / 64.0)))
        .init_resource::<Time<NoDelta>>()
        .init_resource::<Time<SemiFixed>>()
        .add_systems(Update, (no_delta, variable_delta, semi_fixed))
        .add_systems(FixedUpdate, fixed);
}

fn no_delta(world: &mut World) {
    let delta = world.resource::<SimulationDelta>().0;
    let mut time = world.resource_mut::<Time<NoDelta>>();
    time.advance_by(delta);

    *world.resource_mut::<Time>() = time.as_generic();
    world.run_schedule(NoDelta);
    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}

fn variable_delta(world: &mut World) {
    world.run_schedule(VariableDelta);
}

fn semi_fixed(world: &mut World) {
    let mut delta = world.resource::<Time<Virtual>>().delta();
    let timestep = world.resource::<SimulationDelta>().0;

    // Copy of the fixed timestep logic, plus the overstep logic
    world.schedule_scope(SemiFixed, |world, schedule| {
        loop {
            let (timestep, is_overstep) = if let Some(remainder) = delta.checked_sub(timestep) {
                delta = remainder;
                (timestep, false)
            } else {
                (delta, true)
            };
            let mut time = world.resource_mut::<Time<SemiFixed>>();
            time.advance_by(timestep);
            *world.resource_mut::<Time>() = time.as_generic();
            schedule.run(world);
            if is_overstep {
                break;
            }
        }
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}

fn fixed(world: &mut World) {
    world.run_schedule(Fixed);
}
