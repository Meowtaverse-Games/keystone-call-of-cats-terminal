use bevy::prelude::*;
use bevy_window_reveal::{WindowRevealConfig, WindowRevealPlugin};

mod app;
mod scenes;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WindowRevealPlugin(WindowRevealConfig::default()))
        .add_systems(Startup, setup)
        .add_plugins(app::GamePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
