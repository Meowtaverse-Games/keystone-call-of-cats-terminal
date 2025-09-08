
use bevy::prelude::*;
use crate::{app::AppState, scenes::playing::components::PlayingScene};

#[derive(Component)]
pub struct Player;

pub fn setup_game(mut commands: Commands) {
    commands.spawn(PlayingScene)
        .with_children(|p| {
        p.spawn((Sprite::from_color(Color::srgb(0.2,0.6,1.0), Vec2::splat(64.0)),
                 Transform::from_xyz(0.0, 0.0, 0.0),
                 Player));
    });
}

pub fn gameplay(mut q: Query<&mut Transform, With<Player>>, time: Res<Time>) {
    if let Ok(mut t) = q.single_mut() {
        t.translation.x += 100.0 * time.delta_secs();
    }
}

pub fn back_to_menu(
    kb: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if kb.just_pressed(KeyCode::Escape) {
        next.set(AppState::Playing);
    }
}
