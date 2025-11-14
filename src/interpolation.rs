//! Visual smoothing for fixed timestep, to address the stuttering.

use crate::timestep::Fixed;
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

#[derive(Component, Default, Deref, DerefMut)]
#[component(on_add = init_other_transforms)]
pub struct SimulationTransform(Transform);

impl SimulationTransform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Transform::from_xyz(x, y, z))
    }
}

#[derive(Component, Deref, DerefMut)]
struct PreviousTransform(Transform);

fn init_other_transforms(mut world: DeferredWorld, context: HookContext) {
    let &SimulationTransform(transform) = world.get(context.entity).unwrap();
    world
        .commands()
        .entity(context.entity)
        .insert((transform, PreviousTransform(transform)));
}

#[derive(Resource, Component, Default, Clone, Copy)]
pub enum InterpolationMode {
    /// No interpolation
    None,
    /// Save the transform value of the previous simulation update, and interpolate between that and the current value
    #[default]
    Interpolate,
    /// Save the transform value of the previous simulation update, and use it to extrapolate beyond the current value
    Extrapolate,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<InterpolationMode>()
        .add_systems(PostUpdate, (update_non_fixed, interpolate_transforms))
        .add_systems(FixedPreUpdate, update_previous_transforms);
}

fn update_non_fixed(mut non_fixed: Query<(&mut Transform, &SimulationTransform), Without<Fixed>>) {
    // Every other timestep mode besides Fixed is guaranteed to update at least once per render frame, and thus doesn't suffer
    // from stuttering like Fixed does. So, these modes don't need any interpolation.
    for (mut render, simulation) in non_fixed.iter_mut() {
        *render = simulation.0;
    }
}

fn interpolate_transforms(
    mut fixed: Query<(&mut Transform, &PreviousTransform, &SimulationTransform), With<Fixed>>,
    mode: Res<InterpolationMode>,
    time: Res<Time<bevy::prelude::Fixed>>,
) {
    let t = match *mode {
        InterpolationMode::None => 1.0,
        InterpolationMode::Interpolate => time.overstep_fraction(),
        InterpolationMode::Extrapolate => time.overstep_fraction() + 1.0,
    };

    for (mut render, previous, simulation) in fixed.iter_mut() {
        *render = Transform {
            translation: previous.translation.lerp(simulation.translation, t),
            rotation: previous.rotation.slerp(simulation.rotation, t),
            scale: previous.scale.lerp(simulation.scale, t),
        };
    }
}

fn update_previous_transforms(
    mut fixed: Query<(&mut PreviousTransform, &SimulationTransform), With<Fixed>>,
) {
    for (mut previous, current) in fixed.iter_mut() {
        previous.0 = current.0;
    }
}
