use bevy::prelude::*;
use crate::{app::AppState, scenes::playing::components::PlayingScene};
use super::systems::*;

pub struct PlayingPlugin;
impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_game)
           .add_systems(Update,   (gameplay, back_to_menu).run_if(in_state(AppState::Playing)))
           .add_systems(OnExit(AppState::Playing), cleanup);
    }
}

fn cleanup(mut commands: Commands, q: Query<Entity, With<PlayingScene>>) {
    for item in q.iter() {
        commands.entity(item).despawn();
    }
}
