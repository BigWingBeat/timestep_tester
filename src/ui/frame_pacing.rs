use bevy::{
    feathers::controls::radio,
    prelude::*,
    ui::Checked,
    ui_widgets::{RadioGroup, ValueChange, observe},
    window::PresentMode,
};

use crate::{configuration::respawn, ui::WindowPresentMode};

pub fn presentation_modes() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
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
                        commands.run_system_cached(respawn);
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            },
        ),
        children![
            Text::new("Switch Window Presentation Mode"),
            (radio(
                WindowPresentMode(PresentMode::AutoVsync),
                Spawn(Text::new("AutoVsync (FifoRelaxed -> Fifo)"))
            )),
            (radio(
                WindowPresentMode(PresentMode::AutoNoVsync),
                Spawn(Text::new("AutoNoVsync (Immediate -> Mailbox -> Fifo)"))
            )),
            (radio(
                WindowPresentMode(PresentMode::Fifo),
                Spawn(Text::new("Fifo"))
            )),
            (radio(
                WindowPresentMode(PresentMode::FifoRelaxed),
                Spawn(Text::new("FifoRelaxed"))
            )),
            (radio(
                WindowPresentMode(PresentMode::Immediate),
                Spawn(Text::new("Immediate"))
            )),
            (radio(
                (Checked, WindowPresentMode(PresentMode::Mailbox)),
                Spawn(Text::new("Mailbox"))
            )),
        ],
    )
}
