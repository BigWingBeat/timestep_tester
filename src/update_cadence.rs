//! Modified version of the dev tools frame time visualiser,
//! for showing how many times in a frame a given schedule has run.

mod update_cadence_graph;
use update_cadence_graph::*;

use std::collections::VecDeque;

use bevy::{
    dev_tools::fps_overlay::FpsOverlayConfig,
    diagnostic::FrameTimeDiagnosticsPlugin,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    platform::collections::HashMap,
    prelude::*,
    render::storage::ShaderStorageBuffer,
};

/// [`GlobalZIndex`] used to render the overlay.
///
/// We use a number slightly under `i32::MAX` so you can render on top of it if you really need to.
pub const FPS_OVERLAY_ZINDEX: i32 = i32::MAX - 32;

// Used to scale the update cadence graph based on the fps text size
const UPDATE_CADENCE_GRAPH_WIDTH_SCALE: f32 = 9.6;
const UPDATE_CADENCE_GRAPH_HEIGHT_SCALE: f32 = 3.2;

const MAX_HISTORY_LENGTH: usize = 120;

/// A plugin that adds an update cadence overlay to the Bevy application.
///
/// Visualises how many times each frame the given schedule is ran.
///
/// By default, provides a graph for the FixedUpdate schedule.
///
/// This plugin will add the [`FrameTimeDiagnosticsPlugin`] if it wasn't added before.
pub struct UpdateCadencePlugin {
    /// Starting configuration of overlays, this can be later be changed through [`FpsOverlayConfig`] resource.
    pub configs: Vec<UpdateCadenceConfig>,
}

impl UpdateCadencePlugin {
    /// No graphs for no schedules.
    pub fn new() -> Self {
        Self { configs: default() }
    }

    /// Add a graph for the given schedule.
    pub fn add_schedule(mut self, schedule: impl ScheduleLabel) -> Self {
        self.configs.push(UpdateCadenceConfig::new(schedule));
        self
    }

    /// Add a graph with the given configuration.
    pub fn add_schedule_config(mut self, config: UpdateCadenceConfig) -> Self {
        self.configs.push(config);
        self
    }
}

impl Default for UpdateCadencePlugin {
    fn default() -> Self {
        Self {
            configs: vec![UpdateCadenceConfig::new(FixedUpdate)],
        }
    }
}

impl Plugin for UpdateCadencePlugin {
    fn build(&self, app: &mut App) {
        // TODO: Use plugin dependencies, see https://github.com/bevyengine/bevy/issues/69
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        if !app.is_plugin_added::<UpdateCadenceGraphPlugin>() {
            app.add_plugins(UpdateCadenceGraphPlugin);
        }

        for config in &self.configs {
            app.add_systems(config.schedule, update_count(config.schedule));
        }

        app.insert_resource(ScheduleUpdateCounts(
            self.configs
                .iter()
                .map(|config| (config.schedule, VecDeque::new()))
                .collect(),
        ))
        .insert_resource(UpdateCadenceConfigs(self.configs.clone()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_display,
                customize_overlay,
                update_max_updates,
                update_schedule_name,
            ),
        )
        .add_systems(Last, flush_counts);
    }
}

#[derive(Resource, Default)]
struct ScheduleUpdateCounts(HashMap<Interned<dyn ScheduleLabel>, VecDeque<u32>>);

/// Not pub so don't need to worry about it changing at runtime
#[derive(Resource)]
struct UpdateCadenceConfigs(Vec<UpdateCadenceConfig>);

/// Configuration options for the update cadence overlay.
#[derive(Component, Clone)]
pub struct UpdateCadenceConfig {
    /// The schedule being visualised.
    pub schedule: Interned<dyn ScheduleLabel>,
    /// Configuration of text in the overlay.
    pub text_config: TextFont,
    /// Color of text in the overlay.
    pub text_color: Color,
    /// Displays the FPS overlay if true.
    pub enabled: bool,
    /// Configuration of the update cadence graph
    pub update_cadence_graph_config: UpdateCadenceGraphConfig,
}

impl UpdateCadenceConfig {
    fn new(schedule: impl ScheduleLabel) -> Self {
        Self {
            schedule: schedule.intern(),
            text_config: TextFont {
                font: Handle::<Font>::default(),
                font_size: 20.0,
                ..default()
            },
            text_color: Color::WHITE,
            enabled: true,
            update_cadence_graph_config: UpdateCadenceGraphConfig::target_fps(60.0),
        }
    }
}

/// Configuration of the update cadence graph
#[derive(Clone, Copy)]
pub struct UpdateCadenceGraphConfig {
    /// Is the graph visible
    pub enabled: bool,
    /// The minimum acceptable FPS
    ///
    /// Anything below this will show a red bar
    pub min_fps: f32,
    /// The target FPS
    ///
    /// Anything above this will show a green bar
    pub target_fps: f32,
    /// The max number of times the schedule should run in a frame
    pub max_schedule_runs: u32,
}

impl UpdateCadenceGraphConfig {
    /// Constructs a default config for a given target fps
    pub fn target_fps(target_fps: f32) -> Self {
        Self {
            target_fps,
            ..default()
        }
    }
}

impl Default for UpdateCadenceGraphConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_fps: 30.0,
            target_fps: 60.0,
            max_schedule_runs: 3,
        }
    }
}

#[derive(Component)]
struct ScheduleNameText;

#[derive(Component)]
struct RecentMaxUpdatesText;

#[derive(Component)]
struct UpdateCadenceGraph;

fn setup(
    mut commands: Commands,
    is_fps_overlay_present: Option<Res<FpsOverlayConfig>>,
    overlay_configs: Res<UpdateCadenceConfigs>,
    mut update_cadence_graph_materials: ResMut<Assets<UpdateCadenceGraphMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    const MARGIN: f32 = 64.0;

    let mut offset = 0.0;
    if let Some(fps_config) = is_fps_overlay_present {
        // Should be kept in sync with the same const in Bevy
        const FRAME_TIME_GRAPH_WIDTH_SCALE: f32 = 6.0;
        let fps_overlay_width = fps_config.text_config.font_size * FRAME_TIME_GRAPH_WIDTH_SCALE;
        offset += fps_overlay_width + MARGIN;
    }

    for overlay_config in &overlay_configs.0 {
        let font_size = overlay_config.text_config.font_size;
        let width = font_size * UPDATE_CADENCE_GRAPH_WIDTH_SCALE;
        commands
            .spawn((
                Node {
                    // We need to make sure the overlay doesn't affect the position of other UI nodes
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    left: Val::Px(offset),
                    ..default()
                },
                // Render overlay on top of everything
                GlobalZIndex(FPS_OVERLAY_ZINDEX),
                Pickable::IGNORE,
                overlay_config.clone(),
            ))
            .with_children(|p| {
                p.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    Text::default(),
                    overlay_config.text_config.clone(),
                    TextColor(overlay_config.text_color),
                    Pickable::IGNORE,
                ))
                .with_children(|p| {
                    p.spawn((TextSpan::new("Cadence: "), Pickable::IGNORE))
                        .with_child((TextSpan::default(), ScheduleNameText));

                    p.spawn(TextSpan::new('\n'));

                    p.spawn((TextSpan::new("Recent max: "), Pickable::IGNORE))
                        .with_child((TextSpan::default(), RecentMaxUpdatesText));
                });

                p.spawn((
                    Node {
                        width: Val::Px(width),
                        height: Val::Px(font_size * UPDATE_CADENCE_GRAPH_HEIGHT_SCALE),
                        display: if overlay_config.update_cadence_graph_config.enabled {
                            Display::DEFAULT
                        } else {
                            Display::None
                        },
                        ..default()
                    },
                    Pickable::IGNORE,
                    MaterialNode::from(update_cadence_graph_materials.add(
                        UpdateCadenceGraphMaterial {
                            dts: buffers.add(ShaderStorageBuffer {
                                // Initialize with dummy data because the default (`data: None`) will
                                // cause a panic in the shader if the update cadence graph is constructed
                                // with `enabled: false`.
                                data: Some(vec![0, 0, 0, 0]),
                                ..default()
                            }),
                            counts: buffers.add(ShaderStorageBuffer {
                                data: None,
                                ..default()
                            }),
                            config: UpdateCadenceGraphConfigUniform::new(
                                overlay_config.update_cadence_graph_config.target_fps,
                                overlay_config.update_cadence_graph_config.min_fps,
                                overlay_config.update_cadence_graph_config.max_schedule_runs,
                                true,
                            ),
                        },
                    )),
                    UpdateCadenceGraph,
                ));
            });
        offset += width + MARGIN;
    }
}

fn update_count(schedule: impl ScheduleLabel) -> impl IntoSystem<(), (), ()> {
    let schedule = schedule.intern();
    IntoSystem::into_system(move |mut counts: ResMut<ScheduleUpdateCounts>| {
        let counts = counts.0.entry(schedule).or_default();
        if let Some(count) = counts.back_mut() {
            *count += 1;
        }
    })
}

fn flush_counts(mut counts: ResMut<ScheduleUpdateCounts>) {
    for counts in counts.0.values_mut() {
        if counts.len() >= MAX_HISTORY_LENGTH {
            counts.pop_front();
        }
        counts.push_back(0);
    }
}

fn update_max_updates(
    query: Query<(&UpdateCadenceConfig, &Children)>,
    is_text_root: Query<Has<Text>>,
    is_max_updates: Query<Has<RecentMaxUpdatesText>>,
    update_counts: Res<ScheduleUpdateCounts>,
    mut writer: TextUiWriter,
) {
    for (overlay_config, children) in query.iter() {
        for &entity in children {
            if let Ok(true) = is_text_root.get(entity) {
                writer.for_each(entity, |entity, _, mut text, _, _| {
                    if let Ok(true) = is_max_updates.get(entity)
                        && let Some(counts) = update_counts.0.get(&overlay_config.schedule)
                    {
                        let max_updates = counts.iter().max().copied().unwrap_or_default();
                        *text = format!("{max_updates}");
                    }
                });
            }
        }
    }
}

fn update_schedule_name(
    query: Query<(&UpdateCadenceConfig, &Children), Changed<UpdateCadenceConfig>>,
    is_text_root: Query<Has<Text>>,
    is_schedule_name: Query<Has<ScheduleNameText>>,
    mut writer: TextUiWriter,
) {
    for (overlay_config, children) in query.iter() {
        for &entity in children {
            if let Ok(true) = is_text_root.get(entity) {
                writer.for_each(entity, |entity, _, mut text, _, _| {
                    if let Ok(true) = is_schedule_name.get(entity) {
                        *text = format!("{:?}", overlay_config.schedule);
                    }
                });
            }
        }
    }
}

fn customize_overlay(
    query: Query<(&UpdateCadenceConfig, &Children), Changed<UpdateCadenceConfig>>,
    is_text_root: Query<Has<Text>>,
    mut writer: TextUiWriter,
) {
    for (overlay_config, children) in query.iter() {
        for &entity in children {
            if let Ok(true) = is_text_root.get(entity) {
                writer.for_each_font(entity, |mut font| {
                    *font = overlay_config.text_config.clone();
                });
                writer.for_each_color(entity, |mut color| color.0 = overlay_config.text_color);
            }
        }
    }
}

fn toggle_display(
    query: Query<(&UpdateCadenceConfig, &Children), Changed<UpdateCadenceConfig>>,
    mut text_node: Query<&mut Node, (With<Text>, Without<UpdateCadenceGraph>)>,
    mut graph_node: Query<&mut Node, (With<UpdateCadenceGraph>, Without<Text>)>,
) {
    for (overlay_config, children) in query.iter() {
        for &entity in children {
            if let Ok(mut text_node) = text_node.get_mut(entity) {
                if overlay_config.enabled {
                    text_node.display = Display::DEFAULT;
                } else {
                    text_node.display = Display::None;
                }
            }

            if let Ok(mut graph_node) = graph_node.get_mut(entity) {
                if overlay_config.update_cadence_graph_config.enabled {
                    // Scale the update cadence graph based on the font size of the overlay
                    let font_size = overlay_config.text_config.font_size;
                    graph_node.width = Val::Px(font_size * UPDATE_CADENCE_GRAPH_WIDTH_SCALE);
                    graph_node.height = Val::Px(font_size * UPDATE_CADENCE_GRAPH_HEIGHT_SCALE);

                    graph_node.display = Display::DEFAULT;
                } else {
                    graph_node.display = Display::None;
                }
            }
        }
    }
}
