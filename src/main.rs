use bevy::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(DefaultPlugins).run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
