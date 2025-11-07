use std::{marker::PhantomData, ptr};

use bevy::{
    ecs::{
        component::{Immutable, StorageType},
        lifecycle::{ComponentHook, HookContext},
        world::DeferredWorld,
    },
    feathers::{
        self,
        controls::{ButtonProps, ButtonVariant, button},
        dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners,
        theme::UiTheme,
    },
    input_focus::tab_navigation::TabGroup,
    prelude::*,
    ui::Checked,
    ui_widgets::{Button, RadioButton, RadioGroup, ValueChange, observe},
};

mod presentation_modes;
mod simulation;
mod timesteps;
mod update_rate;

pub use simulation::SimulationDescription;

use crate::ui::{
    presentation_modes::presentation_modes, simulation::simulation, timesteps::timesteps,
    update_rate::update_rate,
};

const GAP_SIZE: Val = Val::Px(12.0);
const MAX_WIDTH: Val = Val::Px(720.0);

/// Add a description below a node
fn describe(node: impl Bundle, description: impl Into<String>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            node,
            (Text::new(description), TextFont::from_font_size(18.0)),
        ],
    )
}

/// Hack
struct Remove<T>(PhantomData<T>);

impl<T: Bundle> Component for Remove<T> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    type Mutability = Immutable;

    fn on_insert() -> Option<ComponentHook> {
        Some(|mut world: DeferredWorld, context: HookContext| {
            world
                .commands()
                .entity(context.entity)
                .remove::<(T, Self)>();
        })
    }
}

fn remove<T>() -> Remove<T> {
    Remove(PhantomData)
}

#[derive(Component)]
struct TabName(&'static str);

impl PartialEq for TabName {
    fn eq(&self, other: &Self) -> bool {
        // Reference (address) equality
        ptr::eq(self.0, other.0)
    }
}

impl Eq for TabName {}

macro_rules! tabs {
	[$(($tab:literal, $node:expr)), *$(,)?] => {{
        children![
        (
            Node::default(),
            RadioGroup,
            observe(
                |on: On<ValueChange<Entity>>,
                 mut radios: Query<(Entity, &mut ButtonVariant, &TabName)>,
                 mut tabs: Query<(&mut Node, &TabName), Without<RadioButton>>,
                 mut commands: Commands| {
                    for (entity, mut variant, radio_name) in radios.iter_mut() {
                        if entity == on.value {
                            *variant = ButtonVariant::Primary;
                            commands.entity(entity).insert(Checked);
                            for (mut node, tab_name) in tabs.iter_mut() {
                                if radio_name == tab_name {
                                    node.display = Display::Flex;
                                } else {
                                    node.display = Display::None;
                                }
                            }
                        } else {
                            *variant = ButtonVariant::Normal;
                            commands.entity(entity).remove::<Checked>();
                        }
                    }
                },
            ),
            {
                let [first, middle @ .., last] = [$($tab),*];
                let [first_name, middle_name @ .., last_name] = [$(TabName($tab)),*];
                Children::spawn((
                    Spawn(button(
                        ButtonProps {
                            variant: ButtonVariant::Primary,
                            corners: RoundedCorners::TopLeft,
                        },
                        (RadioButton, remove::<Button>(), first_name),
                        Spawn(Text::new(first)),
                    )),
                    SpawnIter(middle.into_iter().zip(middle_name).map(|(middle, name)| {
                        button(
                            ButtonProps {
                                corners: RoundedCorners::None,
                                ..default()
                            },
                            (RadioButton, remove::<Button>(), name),
                            Spawn(Text::new(middle)),
                        )
                    })),
                    Spawn(button(
                        ButtonProps {
                            corners: RoundedCorners::TopRight,
                            ..default()
                        },
                        (RadioButton, remove::<Button>(), last_name),
                        Spawn(Text::new(last)),
                    )),
                ))
            },
        ),
        (
            Node {
                padding: UiRect::all(GAP_SIZE),
                border: UiRect::all(Val::Px(2.0)).with_top(Val::ZERO),
                max_width: MAX_WIDTH,
                ..default()
            },
            BackgroundColor(feathers::palette::GRAY_1),
            BorderColor::all(feathers::palette::WARM_GRAY_1),
            BorderRadius::all(Val::Px(4.0)).with_top(Val::ZERO),
            {
                let mut children = ($(Spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: GAP_SIZE,
                        display: Display::None,
                        ..default()
                    },
                    $node,
                    TabName($tab),
                ))),*);

                // Show the first (active by default) tab
                children.0.0.0.display = Display::Flex;

                Children::spawn(children)
            },
        ),
    ]
    }};
}

pub fn plugin(app: &mut App) {
    app.insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ClearColor(feathers::palette::GRAY_0))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: GAP_SIZE,
            top: GAP_SIZE * 10.0,
            max_width: MAX_WIDTH,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        TabGroup::default(),
        tabs![
            ("Simulation", simulation()),
            ("Timesteps", timesteps()),
            ("Presentation Modes", presentation_modes()),
            ("Update Rate", update_rate()),
        ],
    ));
}
