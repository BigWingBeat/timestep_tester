use bevy::{
    feathers::controls::radio,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange, observe},
    window::{PresentMode, WindowMode},
};

use crate::ui::{GAP_SIZE, describe};

#[derive(Component)]
struct WindowPresentMode(PresentMode);

#[derive(Component, Clone, Copy)]
enum FullscreenMode {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

impl From<FullscreenMode> for WindowMode {
    fn from(mode: FullscreenMode) -> Self {
        match mode {
            FullscreenMode::Windowed => Self::Windowed,
            FullscreenMode::BorderlessFullscreen => {
                Self::BorderlessFullscreen(MonitorSelection::Current)
            }
            FullscreenMode::Fullscreen => {
                Self::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current)
            }
        }
    }
}

pub fn presentation_modes() -> impl Bundle {
    children![
        (
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: GAP_SIZE,
                ..default()
            },
            RadioGroup,
            observe(
                |on: On<ValueChange<Entity>>,
                 radios: Query<(Entity, &WindowPresentMode)>,
                 mut windows: Query<&mut Window>,
                 mut commands: Commands| {
                    for (entity, mode) in radios.iter() {
                        if entity == on.value {
                            commands.entity(entity).insert(Checked);
                            for mut window in windows.iter_mut() {
                                window.present_mode = mode.0;
                            }
                        } else {
                            commands.entity(entity).remove::<Checked>();
                        }
                    }
                },
            ),
            children![
                describe(
                    Text::new("Switch Window Presentation Mode:"),
                    "See wgpu::PresentMode for more information."
                ),
                describe(
                    radio(
                        WindowPresentMode(PresentMode::AutoVsync),
                        Spawn(Text::new("AutoVsync"))
                    ),
                    "Chooses FifoRelaxed -> Fifo based on availability."
                ),
                describe(
                    radio(
                        WindowPresentMode(PresentMode::AutoNoVsync),
                        Spawn(Text::new("AutoNoVsync"))
                    ),
                    "Chooses Immediate -> Mailbox -> Fifo based on availability."
                ),
                describe(
                    radio(
                        WindowPresentMode(PresentMode::Fifo),
                        Spawn(Text::new("Fifo"))
                    ),
                    "Vsync with a ~3 frame queue. Adds latency, no screen tearing."
                ),
                describe(
                    radio(
                        WindowPresentMode(PresentMode::FifoRelaxed),
                        Spawn(Text::new("FifoRelaxed"))
                    ),
                    "\"Adaptive\" Vsync with a ~3 frame queue. Adds latency, causes screen tearing."
                ),
                describe(
                    radio(
                        WindowPresentMode(PresentMode::Immediate),
                        Spawn(Text::new("Immediate"))
                    ),
                    "No vsync, no frame queue. Low latency, causes screen tearing."
                ),
                describe(
                    radio(
                        (Checked, WindowPresentMode(PresentMode::Mailbox)),
                        Spawn(Text::new("Mailbox"))
                    ),
                    "\"Fast\" vsync, no frame queue. Low latency, no screen tearing."
                ),
            ],
        ),
        (
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: GAP_SIZE,
                ..default()
            },
            RadioGroup,
            observe(
                |on: On<ValueChange<Entity>>,
                 radios: Query<(Entity, &FullscreenMode)>,
                 mut windows: Query<&mut Window>,
                 mut commands: Commands| {
                    for (entity, &mode) in radios.iter() {
                        if entity == on.value {
                            commands.entity(entity).insert(Checked);
                            for mut window in windows.iter_mut() {
                                window.mode = mode.into();
                            }
                        } else {
                            commands.entity(entity).remove::<Checked>();
                        }
                    }
                },
            ),
            children![
                describe(
                    Text::new("Switch Fullscreen mode:"),
                    "See bevy::window::WindowMode for more information."
                ),
                radio(
                    (Checked, FullscreenMode::Windowed),
                    Spawn(Text::new("Windowed"))
                ),
                radio(
                    FullscreenMode::BorderlessFullscreen,
                    Spawn(Text::new("Borderless Fullscreen"))
                ),
                radio(FullscreenMode::Fullscreen, Spawn(Text::new("Fullscreen"))),
            ],
        )
    ]
}
