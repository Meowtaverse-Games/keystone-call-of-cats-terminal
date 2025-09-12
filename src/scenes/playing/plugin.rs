use super::systems::*;
use crate::app::AppState;
use bevy::prelude::*;

pub struct PlayingPlugin;
impl Plugin for PlayingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), setup_terminal)
            .add_systems(
                Update,
                (
                    handle_received_characters,
                    handle_special_keys, // Backspace / Enter
                    update_pulse_and_layout,
                    update_cursor_hud,
                    handle_window_resize,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), cleanup);
    }
}
