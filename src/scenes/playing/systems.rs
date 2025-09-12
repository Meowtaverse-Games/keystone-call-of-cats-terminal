use bevy::audio::PlaybackMode;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::scenes::playing::components::PlayingScene;

const COLS: usize = 80;
const ROWS: usize = 40;

const PULSE_SECS: f32 = 0.22;
const PULSE_SCALE_MAX: f32 = 1.7;

#[derive(Resource, Clone, Copy, Debug)]
pub struct GridGeom {
    cols: usize,
    rows: usize,
    cell_w: f32,
    cell_h: f32,
    font_size: f32,
    /// レイアウト時の左上原点（ワールド座標）
    origin_top_left: Vec2,
}

/// カーソル位置＋文字エンティティのバッファ（固定長）
#[derive(Resource)]
pub struct Terminal {
    cursor_col: usize,
    cursor_row: usize,
    cells: Vec<Option<Entity>>, // ROWS*COLS
    font: Handle<Font>,
    keySound: Handle<AudioSource>,
}

/// 1文字エンティティ
#[derive(Component)]
pub struct CellChar {
    col: usize,
    row: usize,
}

/// 拡大→縮小のパルス演出用
#[derive(Component)]
pub struct Pulse {
    t: f32, // 0.0..=1.0
}

/// カーソル位置表示用のHUD
#[derive(Component)]
pub struct CursorHud;

pub fn setup_terminal(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/JF-Dot-MPlus12.ttf");
    let keySound: Handle<AudioSource> = asset_server.load("sounds/key/key.wav");

    let window = q_window.single().unwrap();
    let geom = compute_grid_geom(window.width(), window.height());
    commands.insert_resource(geom);

    let cells = vec![None; COLS * ROWS];

    commands.insert_resource(Terminal {
        cursor_col: 0,
        cursor_row: 0,
        cells,
        font,
        keySound,
    });

    // カーソル位置HUD（左上に小さく表示）
    let hud_pos = Vec2::new(geom.origin_top_left.x + 8.0, geom.origin_top_left.y - 8.0);
    commands.spawn((
        Text2d(format!("({},{})", 0, 0)),
        TextFont {
            font: font.clone(),
            font_size: (geom.font_size * 0.8).max(8.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Transform {
            translation: Vec3::new(hud_pos.x, hud_pos.y, 10.0),
            ..default()
        },
        Visibility::Visible,
        CursorHud,
        Name::new("Cursor HUD"),
    ));
}

/// ウィンドウサイズからセルサイズ・フォントサイズを算出
pub fn compute_grid_geom(w: f32, h: f32) -> GridGeom {
    // セルの理想サイズ
    let cell_w = w / COLS as f32;
    let cell_h = h / ROWS as f32;

    // 等幅フォントの幅 ~ font_size * 0.6 程度（目安）
    // 横方向と縦方向のどちらにも収まるよう font_size を決める
    let font_size_from_h = cell_h * 0.6;
    let font_size_from_w = cell_w * 1.0; // 0.7 * 0.5;
    let font_size = font_size_from_h.min(font_size_from_w).max(8.0);

    // 左上原点（ワールド座標）
    let origin_top_left = Vec2::new(-w * 0.5, h * 0.5);

    GridGeom {
        cols: COLS,
        rows: ROWS,
        cell_w,
        cell_h,
        font_size,
        origin_top_left,
    }
}

/// 文字入力：Printable を受ける。制御文字は除外。
pub fn handle_received_characters(
    mut keys: EventReader<KeyboardInput>,
    mut commands: Commands,
    mut term: ResMut<Terminal>,
    geom: Res<GridGeom>,
) {
    for key in keys.read() {
        if !key.state.is_pressed() {
            continue;
        }

        if let bevy::input::keyboard::Key::Character(ch) = &key.logical_key {
            if ch.is_ascii() == false {
                continue;
            }
            ch.chars()
                .next()
                .map(|c| if c.is_control() { None } else { Some(c) })
                .flatten()
                .map(|c| {
                    spawn_char_entity(&mut commands, &mut term, *geom, c);
                });
        }
    }
}

/// Backspace / Enter 処理
pub fn handle_special_keys(
    mut keys: EventReader<KeyboardInput>,
    mut term: ResMut<Terminal>,
    mut commands: Commands,
    geom: Res<GridGeom>,
) {
    for key in keys.read() {
        if !key.state.is_pressed() {
            continue;
        }
        match key.key_code {
            KeyCode::Enter => {
                new_line(&mut term);
            }
            KeyCode::Backspace => {
                backspace(&mut commands, &mut term);
            }
            _ => {}
        }
    }
}

/// 1文字エンティティ作成＋パルス付与＋カーソル前進
pub fn spawn_char_entity(commands: &mut Commands, term: &mut Terminal, geom: GridGeom, ch: char) {
    let (col, row) = (term.cursor_col, term.cursor_row);
    let idx = row * geom.cols + col;

    print!("{}", ch);
    commands.spawn((
        AudioPlayer::new(term.keySound.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));

    // 既存のセルに文字があるなら消す（上書き想定）
    if let Some(e) = term.cells[idx].take() {
        commands.entity(e).despawn();
    }

    let pos = cell_to_world(geom, col, row);

    let e = commands
        .spawn((
            Text2d(ch.to_string()),
            TextFont {
                font: term.font.clone(),
                font_size: geom.font_size,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform {
                translation: Vec3::new(pos.x, pos.y, 0.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            Visibility::Visible,
            CellChar { col, row },
            Pulse { t: 0.0 },
            Name::new(format!("char({},{}): '{}'", col, row, ch)),
        ))
        .id();

    term.cells[idx] = Some(e);
    advance_cursor(term);
}

/// カーソルを1マス進める（末尾なら次の行／最終行末尾ならこれ以上進めない）
pub fn advance_cursor(term: &mut Terminal) {
    term.cursor_col += 1;
    if term.cursor_col >= COLS {
        term.cursor_col = 0;
        term.cursor_row += 1;
    }
    if term.cursor_row >= ROWS {
        // シンプル案：最終行に達したら固定（スクロールは未実装）
        term.cursor_row = ROWS - 1;
        term.cursor_col = COLS - 1;
    }
}

/// 改行
pub fn new_line(term: &mut Terminal) {
    term.cursor_col = 0;
    if term.cursor_row + 1 < ROWS {
        term.cursor_row += 1;
    }
}

/// バックスペース：1文字戻って削除
pub fn backspace(commands: &mut Commands, term: &mut Terminal) {
    if term.cursor_col == 0 && term.cursor_row == 0 {
        return;
    }
    // ひとつ戻る
    if term.cursor_col > 0 {
        term.cursor_col -= 1;
    } else {
        term.cursor_row -= 1;
        term.cursor_col = COLS - 1;
    }
    let idx = term.cursor_row * COLS + term.cursor_col;
    if let Some(e) = term.cells[idx].take() {
        commands.entity(e).despawn_recursive();
    }
}

/// セル座標→ワールド座標（セル中心）
pub fn cell_to_world(geom: GridGeom, col: usize, row: usize) -> Vec2 {
    let x = geom.origin_top_left.x + (col as f32 + 0.5) * geom.cell_w;
    let y = geom.origin_top_left.y - (row as f32 + 0.5) * geom.cell_h;
    Vec2::new(x, y)
}

/// パルス演出：tを進め、Transform.scale を 1.0→max→1.0 に補間
pub fn update_pulse_and_layout(
    time: Res<Time>,
    geom: Res<GridGeom>,
    mut q: Query<(&mut Transform, &mut Pulse, &CellChar)>,
) {
    for (mut tf, mut pulse, cell) in q.iter_mut() {
        // t進行
        pulse.t = (pulse.t + time.delta_secs() / PULSE_SECS).min(1.0);

        // イージング（out-back風の簡易カーブ）
        let s = ease_out_back(pulse.t, 1.0, PULSE_SCALE_MAX - 1.0);
        tf.scale = Vec3::splat(s);

        // 念のため位置も毎フレーム整える（ウィンドウ拡縮やフォント遅延に耐性）
        let p = cell_to_world(*geom, cell.col, cell.row);
        tf.translation.x = p.x;
        tf.translation.y = p.y;
    }
}

/// カーソル位置HUDの更新（文字列と配置を毎フレーム調整）
pub fn update_cursor_hud(
    geom: Res<GridGeom>,
    term: Res<Terminal>,
    mut q_hud: Query<(&mut Text2d, &mut Transform), With<CursorHud>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok((mut text, mut tf)) = q_hud.get_single_mut() {
        // 表示テキスト更新
        text.0 = format!("cursor: ({},{})", term.cursor_col, term.cursor_row);

        // ウィンドウの左上に沿わせる（8pxマージン）
        if let Ok(w) = q_window.get_single() {
            let g = compute_grid_geom(w.width(), w.height());
            let pos = Vec2::new(g.origin_top_left.x + 8.0, g.origin_top_left.y - 8.0);
            tf.translation.x = pos.x;
            tf.translation.y = pos.y;
        } else {
            // 予備：既存geomから算出
            let pos = Vec2::new(geom.origin_top_left.x + 8.0, geom.origin_top_left.y - 8.0);
            tf.translation.x = pos.x;
            tf.translation.y = pos.y;
        }
    }
}

/// ウィンドウリサイズ対応：セル/フォント再計算
pub fn handle_window_resize(
    q_window: Query<&Window, (Changed<Window>, With<PrimaryWindow>)>,
    mut geom: ResMut<GridGeom>,
) {
    if let Ok(w) = q_window.get_single() {
        *geom = compute_grid_geom(w.width(), w.height());
    }
}

/// 簡易 out-back イージング
/// t:0→1, start, delta(=max-1.0)
pub fn ease_out_back(t: f32, b: f32, c: f32) -> f32 {
    // back係数
    let s = 1.70158;
    let t1 = t - 1.0;
    c * (t1 * t1 * ((s + 1.0) * t1 + s) + 1.0) + b
}

pub fn cleanup(mut commands: Commands, q: Query<Entity, With<PlayingScene>>) {
    for item in q.iter() {
        commands.entity(item).despawn();
    }
}
