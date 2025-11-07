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
        rounded_corners::RoundedCorners,
    },
    prelude::*,
    ui::Checked,
    ui_widgets::{Button, RadioButton, RadioGroup, ValueChange, observe},
};

use crate::ui::{GAP_SIZE, MAX_WIDTH};

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

pub(super) fn tab_buttons<const MIDDLE: usize>(
    first: &'static str,
    middle: [&'static str; MIDDLE],
    last: &'static str,
) -> impl Bundle {
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
            fn tab_button(props: ButtonProps, name: &'static str) -> impl Bundle {
                button(
                    props,
                    (RadioButton, remove::<Button>(), TabName(name)),
                    Spawn(Text::new(name)),
                )
            }

            Children::spawn((
                Spawn(tab_button(
                    ButtonProps {
                        variant: ButtonVariant::Primary,
                        corners: RoundedCorners::TopLeft,
                    },
                    first,
                )),
                SpawnIter(middle.into_iter().map(|middle| {
                    tab_button(
                        ButtonProps {
                            corners: RoundedCorners::None,
                            ..default()
                        },
                        middle,
                    )
                })),
                Spawn(tab_button(
                    ButtonProps {
                        corners: RoundedCorners::TopRight,
                        ..default()
                    },
                    last,
                )),
            ))
        },
    )
}

pub(super) fn tab_contents(tabs: impl Bundle) -> impl Bundle {
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
        tabs,
    )
}

pub(super) fn single_tab(name: &'static str) -> (Node, impl Bundle) {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: GAP_SIZE,
            display: Display::None,
            ..default()
        },
        TabName(name),
    )
}

macro_rules! tabs {
	[$(($tab:literal, $node:expr)), *$(,)?] => {
        Children::spawn((
			Spawn({
				let [first, middle @ .., last] = [$($tab),*];
				crate::ui::tabs::tab_buttons(first, middle, last)
			}),
			Spawn({
				let mut children = ($(Spawn((crate::ui::tabs::single_tab($tab), $node))),*);

				// Show the first (active by default) tab
				children.0.0.0.0.display = Display::Flex;

				let tabs = Children::spawn(children);
				crate::ui::tabs::tab_contents(tabs)
			}),
		))
    };
}

pub(super) use tabs;
