//! Scripted pointer input and screenshots, for checking a UI without touching it (native only).
//!
//! `MUGON_UI_SHOT=out.png` saves a screenshot after a moment and exits (`MUGON_UI_SHOT_FRAME=n` picks
//! the frame, default 150; `MUGON_UI_SHOT_FRAMES=20,40` takes several, written as `out-<frame>.png`).
//!
//! `MUGON_UI_SCRIPT="30:click:60:300;50:drag:250:110:350:110;80:shot:out.png;90:log"` presses the
//! real pointer path (picking) at those frames, so a test exercises the whole widget, observer and
//! state path. A drag moves the pointer in steps. `log` triggers [`ScriptLog`] for the app to print
//! whatever state a test wants to check. Frames run in about 6 ms, so space edits that should become
//! separate undo steps by 100 frames or more. The app exits shortly after the last step.

use bevy::camera::RenderTarget;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::WindowRef;

/// Triggered by the script's `log` step.
#[derive(Event, Clone, Copy, Debug)]
pub struct ScriptLog;

pub struct TestHarnessPlugin;

impl Plugin for TestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (screenshot, script));
    }
}

fn screenshot(mut commands: Commands, mut frame: Local<u32>, mut exit: MessageWriter<AppExit>) {
    let Ok(path) = std::env::var("MUGON_UI_SHOT") else { return };
    let list: Vec<u32> = std::env::var("MUGON_UI_SHOT_FRAMES").ok().map(|value| value.split(',').filter_map(|frame| frame.parse().ok()).collect()).unwrap_or_default();
    let at = std::env::var("MUGON_UI_SHOT_FRAME").ok().and_then(|frame| frame.parse().ok()).unwrap_or(150u32);
    *frame += 1;
    if list.is_empty() && *frame == at {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
    }
    if list.contains(&*frame) {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(format!("{}-{:04}.png", path.trim_end_matches(".png"), *frame)));
    }
    let last = list.iter().copied().max().unwrap_or(at);
    if *frame == last + 20 {
        exit.write(AppExit::Success);
    }
}

fn script(
    mut commands: Commands,
    mut frame: Local<u32>,
    windows: Query<Entity, With<Window>>,
    mut inputs: MessageWriter<PointerInput>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(script) = std::env::var("MUGON_UI_SCRIPT") else { return };
    let Ok(window) = windows.single() else { return };
    *frame += 1;
    let Some(target) = RenderTarget::Window(WindowRef::Entity(window)).normalize(None) else { return };
    let at = |x: f32, y: f32| Location { target: target.clone(), position: Vec2::new(x, y) };
    let mut last = 0;
    for step in script.split(';').filter(|step| !step.is_empty()) {
        let parts: Vec<&str> = step.split(':').collect();
        let start: u32 = parts[0].parse().unwrap_or(0);
        let number = |index: usize| parts.get(index).and_then(|value| value.parse::<f32>().ok()).unwrap_or(0.0);
        last = last.max(start + 10);
        let now = *frame;
        match parts.get(1).copied().unwrap_or_default() {
            "click" if now == start => {
                inputs.write(PointerInput::new(PointerId::Mouse, at(number(2), number(3)), PointerAction::Move { delta: Vec2::ZERO }));
            }
            "click" if now == start + 2 => {
                inputs.write(PointerInput::new(PointerId::Mouse, at(number(2), number(3)), PointerAction::Press(PointerButton::Primary)));
            }
            "click" if now == start + 4 => {
                inputs.write(PointerInput::new(PointerId::Mouse, at(number(2), number(3)), PointerAction::Release(PointerButton::Primary)));
            }
            "drag" if now >= start && now <= start + 12 => {
                let (from, to) = (Vec2::new(number(2), number(3)), Vec2::new(number(4), number(5)));
                let progress = ((now - start) as f32 - 2.0).clamp(0.0, 8.0) / 8.0;
                let position = from.lerp(to, progress);
                let action = match now - start {
                    0 => PointerAction::Move { delta: Vec2::ZERO },
                    2 => PointerAction::Press(PointerButton::Primary),
                    12 => PointerAction::Release(PointerButton::Primary),
                    _ => PointerAction::Move { delta: (to - from) / 8.0 },
                };
                inputs.write(PointerInput::new(PointerId::Mouse, at(position.x, position.y), action));
            }
            "shot" if now == start => {
                // The observer outlives this frame's script string, so the path has to be owned.
                #[allow(clippy::unnecessary_to_owned)]
                let path = parts.get(2).copied().unwrap_or("shot.png").to_string();
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            }
            "log" if now == start => {
                commands.trigger(ScriptLog);
            }
            _ => {}
        }
    }
    if *frame == last + 20 {
        exit.write(AppExit::Success);
    }
}
