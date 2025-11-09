use std::time::Duration;

use bevy::{
    ecs::system::{IntoObserverSystem, ObserverSystem},
    feathers::controls::{SliderProps, checkbox, radio, slider},
    prelude::*,
    ui::{Checked, InteractionDisabled},
    ui_widgets::{RadioGroup, SliderValue, ValueChange, observe},
    winit::{UpdateMode, WinitSettings},
};

use crate::ui::{TabCorners, describe, tabs};

#[derive(Clone, Copy)]
enum Focus {
    Focused,
    Unfocused,
}

impl Focus {
    fn mode(self, settings: &mut CachedWinitSettings) -> &mut CachedUpdateMode {
        match self {
            Self::Focused => &mut settings.focused,
            Self::Unfocused => &mut settings.unfocused,
        }
    }
}

#[derive(Component)]
struct Focused;

#[derive(Component)]
struct Unfocused;

struct Continuous;
struct Reactive;

trait FocusType: Component {
    const VALUE: Focus;
    const DESCRIPTION: &'static str;

    fn reactive_widget_disabled() -> impl Bundle;
}

impl FocusType for Focused {
    const VALUE: Focus = Focus::Focused;
    const DESCRIPTION: &'static str =
        "Settings for how the app updates while the window is in focus.";

    fn reactive_widget_disabled() -> impl Bundle {
        (ReactiveConfigWidget, Focused, InteractionDisabled)
    }
}

impl FocusType for Unfocused {
    const VALUE: Focus = Focus::Unfocused;
    const DESCRIPTION: &'static str =
        "Settings for how the app updates while the window is unfocused.";

    fn reactive_widget_disabled() -> impl Bundle {
        (ReactiveConfigWidget, Unfocused)
    }
}

trait DefaultVariantChecked {
    fn radio_checked() -> impl Bundle;
}

impl DefaultVariantChecked for (Focused, Continuous) {
    fn radio_checked() -> impl Bundle {
        (Checked, Focused, UpdateModeVariant::Continuous)
    }
}

impl DefaultVariantChecked for (Focused, Reactive) {
    fn radio_checked() -> impl Bundle {
        (Focused, UpdateModeVariant::Reactive)
    }
}

impl DefaultVariantChecked for (Unfocused, Continuous) {
    fn radio_checked() -> impl Bundle {
        (Unfocused, UpdateModeVariant::Continuous)
    }
}

impl DefaultVariantChecked for (Unfocused, Reactive) {
    fn radio_checked() -> impl Bundle {
        (Checked, Unfocused, UpdateModeVariant::Reactive)
    }
}

#[derive(Component, Clone, Copy)]
enum UpdateModeVariant {
    Continuous,
    Reactive,
}

impl UpdateModeVariant {
    fn toggle_reactive_disabled(self, mut entity: EntityCommands) {
        match self {
            Self::Continuous => entity.insert(InteractionDisabled),
            Self::Reactive => entity.remove::<InteractionDisabled>(),
        };
    }
}

/// `UpdateMode` is an enum that doesn't retain the reactive mode configuration when set to continuous,
/// so we store that configuration here instead to make updating it easier
#[derive(Resource)]
struct CachedWinitSettings {
    focused: CachedUpdateMode,
    unfocused: CachedUpdateMode,
}

impl Default for CachedWinitSettings {
    fn default() -> Self {
        let default = WinitSettings::default();
        Self {
            focused: default.focused_mode.into(),
            unfocused: default.unfocused_mode.into(),
        }
    }
}

struct CachedUpdateMode {
    variant: UpdateModeVariant,
    wait: Duration,
    react_to_device_events: bool,
    react_to_user_events: bool,
    react_to_window_events: bool,
}

impl CachedUpdateMode {
    fn events(&mut self, events: ReactiveEvents) -> &mut bool {
        match events {
            ReactiveEvents::Device => &mut self.react_to_device_events,
            ReactiveEvents::User => &mut self.react_to_user_events,
            ReactiveEvents::Window => &mut self.react_to_window_events,
        }
    }
}

impl From<UpdateMode> for CachedUpdateMode {
    fn from(mode: UpdateMode) -> Self {
        match mode {
            UpdateMode::Continuous => Self {
                variant: UpdateModeVariant::Continuous,
                wait: Duration::from_secs_f32(1.0 / 60.0),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
            UpdateMode::Reactive {
                wait,
                react_to_device_events,
                react_to_user_events,
                react_to_window_events,
            } => Self {
                variant: UpdateModeVariant::Reactive,
                wait,
                react_to_device_events,
                react_to_user_events,
                react_to_window_events,
            },
        }
    }
}

impl From<&CachedUpdateMode> for UpdateMode {
    fn from(mode: &CachedUpdateMode) -> Self {
        match mode.variant {
            UpdateModeVariant::Continuous => UpdateMode::Continuous,
            UpdateModeVariant::Reactive => UpdateMode::Reactive {
                wait: mode.wait,
                react_to_device_events: mode.react_to_device_events,
                react_to_user_events: mode.react_to_user_events,
                react_to_window_events: mode.react_to_window_events,
            },
        }
    }
}

#[derive(Clone, Copy)]
enum ReactiveEvents {
    Device,
    User,
    Window,
}

fn toggle_reactive(
    focus: Focus,
    events: ReactiveEvents,
) -> impl ObserverSystem<ValueChange<bool>, ()> {
    IntoObserverSystem::into_system(
        move |on: On<ValueChange<bool>>,
              mut settings: ResMut<CachedWinitSettings>,
              mut commands: Commands| {
            *focus.mode(&mut settings).events(events) = on.value;

            if on.value {
                commands.entity(on.source).insert(Checked);
            } else {
                commands.entity(on.source).remove::<Checked>();
            }
        },
    )
}

#[derive(Component)]
struct ReactiveConfigWidget;

#[derive(Component, Default)]
struct UpdateModeTabs;

pub fn plugin(app: &mut App) {
    app.init_resource::<CachedWinitSettings>()
        .add_systems(Update, update_winit_settings);
}

pub fn update_rate() -> impl Bundle {
    let (buttons, contents) = tabs![
        UpdateModeTabs,
        TabCorners::Both,
        ("Focused Mode", winit_update_mode::<Focused>()),
        ("Unfocused Mode", winit_update_mode::<Unfocused>()),
    ];

    Children::spawn((
        Spawn(describe(
            Text::new("Update Rate:"),
            "Settings to control the framerate of the whole application. May be capped by the Presentation Mode if VSync is enabled.",
        )),
        Spawn(buttons),
        contents,
    ))
}

fn winit_update_mode<F>() -> impl Bundle
where
    F: FocusType,
    (F, Continuous): DefaultVariantChecked,
    (F, Reactive): DefaultVariantChecked,
{
    let focus = F::VALUE;
    (
        RadioGroup,
        observe(
            move |on: On<ValueChange<Entity>>,
                  radios: Query<(Entity, &UpdateModeVariant), With<F>>,
                  reactive_widgets: Query<Entity, (With<ReactiveConfigWidget>, With<F>)>,
                  mut settings: ResMut<CachedWinitSettings>,
                  mut commands: Commands| {
                for (entity, &variant) in radios.iter() {
                    if entity == on.value {
                        commands.entity(entity).insert(Checked);
                        focus.mode(&mut settings).variant = variant;
                        for entity in reactive_widgets.iter() {
                            variant.toggle_reactive_disabled(commands.entity(entity));
                        }
                    } else {
                        commands.entity(entity).remove::<Checked>();
                    }
                }
            },
        ),
        children![
            Text::new(F::DESCRIPTION),
            describe(
                radio(
                    <(F, Continuous)>::radio_checked(),
                    Spawn(Text::new("Continuous"))
                ),
                "Uncapped update rate. As fast as possible."
            ),
            describe(
                radio(
                    <(F, Reactive)>::radio_checked(),
                    Spawn(Text::new("Reactive"))
                ),
                "Only updates in response to one of the following events."
            ),
            describe(
                Text::new("Update Frequency:"),
                "In the absence of any of the below events, updates at approximately this frequency."
            ),
            slider(
                SliderProps {
                    value: 60.0,
                    min: 1.0,
                    max: 1000.0
                },
                (
                    F::reactive_widget_disabled(),
                    observe(
                        move |on: On<ValueChange<f32>>,
                              mut commands: Commands,
                              mut settings: ResMut<CachedWinitSettings>| {
                            commands.entity(on.source).insert(SliderValue(on.value));
                            focus.mode(&mut settings).wait =
                                Duration::from_secs_f32(on.value.recip());
                        }
                    )
                ),
            ),
            describe(
                checkbox(
                    (
                        Checked,
                        F::reactive_widget_disabled(),
                        observe(toggle_reactive(focus, ReactiveEvents::Device))
                    ),
                    Spawn(Text::new("Device Events"))
                ),
                "Reacts to device events not associated with any particular window, including (but not limited to) any mouse movement anywhere. Reacts even if the window is not in focus."
            ),
            describe(
                checkbox(
                    (
                        Checked,
                        F::reactive_widget_disabled(),
                        observe(toggle_reactive(focus, ReactiveEvents::User))
                    ),
                    Spawn(Text::new("User Events"))
                ),
                "Reacts to custom application-controlled events. Bevy never triggers this by default."
            ),
            describe(
                checkbox(
                    (
                        Checked,
                        F::reactive_widget_disabled(),
                        observe(toggle_reactive(focus, ReactiveEvents::Window))
                    ),
                    Spawn(Text::new("Window Events"))
                ),
                "Reacts to window events, including (but not limited to) the window being moved or resized, or the mouse moving while on top of the window."
            ),
        ],
    )
}

fn update_winit_settings(mut winit: ResMut<WinitSettings>, cached: Res<CachedWinitSettings>) {
    if !cached.is_changed() {
        return;
    }

    winit.focused_mode = (&cached.focused).into();
    winit.unfocused_mode = (&cached.unfocused).into();
}
