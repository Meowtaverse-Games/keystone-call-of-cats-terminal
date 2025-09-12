use bevy::prelude::*;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use bevy::window::CompositeAlphaMode;
use bevy_window_reveal::{WindowRevealConfig, WindowRevealPlugin};

mod app;
mod scenes;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        transparent: true,
                        decorations: true,
                        #[cfg(target_os = "macos")]
                        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                        #[cfg(target_os = "linux")]
                        composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                        visible: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(WindowRevealPlugin(WindowRevealConfig {
            initial_clear: Some(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_plugins(app::GamePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
