use bevy::{
    asset::{load_internal_asset, uuid_handle},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderStorageBuffer,
    },
    shader::ShaderRef,
};

use super::{ScheduleUpdateCounts, UpdateCadenceConfig};

const UPDATE_CADENCE_GRAPH_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("cf734b1b-a4b5-4a2b-b1e0-2af389f9b289");

/// Plugin that sets up everything to render the update cadence graph material
pub struct UpdateCadenceGraphPlugin;

impl Plugin for UpdateCadenceGraphPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            UPDATE_CADENCE_GRAPH_SHADER_HANDLE,
            "update_cadence_graph.wgsl",
            Shader::from_wgsl
        );

        // TODO: Use plugin dependencies, see https://github.com/bevyengine/bevy/issues/69
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            panic!("Requires FrameTimeDiagnosticsPlugin");
        }

        app.add_plugins(UiMaterialPlugin::<UpdateCadenceGraphMaterial>::default())
            .add_systems(Update, update_frame_time_values);
    }
}

/// The config values sent to the update cadence graph shader
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct UpdateCadenceGraphConfigUniform {
    // minimum expected delta time
    dt_min: f32,
    // maximum expected delta time
    dt_max: f32,
    // maximum expected schedule runs in a frame
    count_max: u32,
    // controls whether or not the bars width are proportional to their delta time
    proportional_width: u32,
}

impl UpdateCadenceGraphConfigUniform {
    /// `proportional_width`: controls whether or not the bars width are proportional to their delta time
    pub fn new(target_fps: f32, min_fps: f32, count_max: u32, proportional_width: bool) -> Self {
        // we want an upper limit that is above the target otherwise the bars will disappear
        let dt_min = 1. / (target_fps * 1.2);
        let dt_max = 1. / min_fps;
        Self {
            dt_min,
            dt_max,
            count_max,
            proportional_width: u32::from(proportional_width),
        }
    }
}

/// The material used to render the update cadence graph ui node
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct UpdateCadenceGraphMaterial {
    /// The history of the previous frame times value.
    ///
    /// This should be updated every frame to match the frame time history from the [`DiagnosticsStore`]
    #[storage(0, read_only)]
    pub dts: Handle<ShaderStorageBuffer>, // Vec<f32>,
    /// The history of the previous schedule update counts.
    ///
    /// This should be updated every frame to match the update counts from ??
    #[storage(1, read_only)]
    pub counts: Handle<ShaderStorageBuffer>, // Vec<u32>,
    /// The configuration values used by the shader to control how the graph is rendered
    #[uniform(2)]
    pub config: UpdateCadenceGraphConfigUniform,
}

impl UiMaterial for UpdateCadenceGraphMaterial {
    fn fragment_shader() -> ShaderRef {
        UPDATE_CADENCE_GRAPH_SHADER_HANDLE.into()
    }
}

/// A system that updates the frame time values sent to the update cadence graph
fn update_frame_time_values(
    query: Query<(&UpdateCadenceConfig, &Children)>,
    material_node: Query<&MaterialNode<UpdateCadenceGraphMaterial>>,
    mut update_cadence_graph_materials: ResMut<Assets<UpdateCadenceGraphMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    diagnostics_store: Res<DiagnosticsStore>,
    update_counts: Res<ScheduleUpdateCounts>,
) {
    let Some(frame_time) = diagnostics_store.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) else {
        return;
    };

    let frame_times = frame_time
        .values()
        // convert to millis
        .map(|x| *x as f32 / 1000.0)
        .collect::<Vec<_>>();

    for (config, children) in query.iter() {
        if !config.update_cadence_graph_config.enabled {
            continue;
        }

        for &entity in children {
            if let Ok(material) = material_node.get(entity)
                && let Some(material) = update_cadence_graph_materials.get_mut(&material.0)
            {
                let buffer = buffers.get_mut(&material.dts).unwrap();
                buffer.set_data(frame_times.clone());

                if let Some(counts) = update_counts.0.get(&config.schedule) {
                    let counts = Vec::from(counts.clone());
                    let buffer = buffers.get_mut(&material.counts).unwrap();
                    buffer.set_data(counts);
                }
            }
        }
    }
}
