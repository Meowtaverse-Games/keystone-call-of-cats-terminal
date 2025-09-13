use bevy::prelude::*;

#[derive(Component)]
pub struct PlayingScene;

/// 1文字エンティティ
#[derive(Component)]
pub struct CellChar {
    pub col: usize,
    pub row: usize,
}

/// 拡大→縮小のパルス演出用
#[derive(Component)]
pub struct Pulse {
    pub t: f32, // 0.0..=1.0
}

/// カーソル位置表示用のHUD
#[derive(Component)]
pub struct CursorHud;
