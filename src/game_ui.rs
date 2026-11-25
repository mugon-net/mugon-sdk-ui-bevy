//! Game UI: progress bar, spinner, player badge, scoreboard row, message log, HUD anchors and safe-area insets.

use bevy::feathers::theme::{ThemeBackgroundColor, ThemeBorderColor};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{GlobalZIndex, ScrollPosition, UiTransform};
use bevy::ui_widgets::ScrollArea;

use crate::theme::{GAP, GAP_SMALL, RADIUS, token};
use crate::widgets::{TouchScroll, text_small};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SafeArea>()
            .add_observer(push_log_message)
            .add_systems(Startup, read_safe_area)
            .add_systems(Update, (update_progress, spin, apply_safe_area, read_safe_area.run_if(on_timer_seconds(2.0))));
    }
}

fn on_timer_seconds(seconds: f32) -> impl FnMut(Res<Time>, Local<f32>) -> bool + Clone {
    move |time: Res<Time>, mut elapsed: Local<f32>| {
        *elapsed += time.delta_secs();
        if *elapsed >= seconds {
            *elapsed = 0.0;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------- progress

/// A progress bar: `ProgressBar(0.0..=1.0)` on the track; change the component to move the fill.
#[derive(Component, Default, Clone, Debug, PartialEq)]
pub struct ProgressBar(pub f32);

#[derive(Component, Default, Clone)]
struct ProgressFill;

pub fn progress_bar(fraction: f32) -> impl Scene {
    let bar = ProgressBar(fraction);
    bsn! {
        Node { height: px(8), width: percent(100), border_radius: BorderRadius::MAX }
        ThemeBackgroundColor(token::DIVIDER)
        bar
        Children [
            ProgressFill
            Node { height: percent(100), width: percent(0), border_radius: BorderRadius::MAX }
            ThemeBackgroundColor(token::CHOICE_BORDER_SELECTED)
        ]
    }
}

fn update_progress(bars: Query<(&ProgressBar, &Children), Changed<ProgressBar>>, mut fills: Query<&mut Node, With<ProgressFill>>) {
    for (bar, children) in &bars {
        for &child in children {
            if let Ok(mut node) = fills.get_mut(child) {
                node.width = percent(bar.0.clamp(0.0, 1.0) * 100.0);
            }
        }
    }
}

// ---------------------------------------------------------------- spinner

/// A ring with a dot circling it, for something that is loading.
#[derive(Component, Default, Clone)]
pub struct Spinner;

pub fn spinner(diameter: f32) -> impl Scene {
    let dot = diameter * 0.2;
    bsn! {
        Spinner
        Node {
            width: px(diameter),
            height: px(diameter),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::MAX,
            justify_content: JustifyContent::Center,
        }
        ThemeBorderColor(token::DIVIDER)
        Children [
            Node { width: px(dot), height: px(dot), margin: UiRect::top(px(-dot * 0.6)), border_radius: BorderRadius::MAX }
            ThemeBackgroundColor(token::CHOICE_BORDER_SELECTED)
        ]
    }
}

fn spin(mut spinners: Query<&mut UiTransform, With<Spinner>>, time: Res<Time>) {
    for mut transform in &mut spinners {
        transform.rotation = Rot2::radians(time.elapsed_secs() * 5.0);
    }
}

// ---------------------------------------------------------------- players and scores

/// A player: a round avatar (the picture if there is one, else the first letter) and the name.
pub fn player_badge(name: String, avatar: Option<Handle<Image>>) -> impl Scene {
    let initial = name.chars().next().map(|letter| letter.to_uppercase().to_string()).unwrap_or_default();
    let picture = avatar.map(|image| bsn_list! { Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100) } ImageNode { image: image } });
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(GAP) }
        Children [
            Node { width: px(32), height: px(32), border_radius: BorderRadius::MAX, justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::clip() }
            ThemeBackgroundColor(token::CHOICE_BG_SELECTED)
            Children [ @text_small(initial) -- {picture} ]
            --
            @text_small(name)
        ]
    }
}

/// One line of a scoreboard: rank, name, score. `highlight` marks the local player.
pub fn scoreboard_row(rank: usize, name: String, score: String, highlight: bool) -> impl Scene {
    let background = if highlight { token::CHOICE_BG_SELECTED } else { token::CHOICE_BG };
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(GAP), padding: UiRect::axes(px(GAP), px(GAP_SMALL)), border_radius: BorderRadius::all(px(RADIUS)) }
        ThemeBackgroundColor(background)
        Children [
            Node { width: px(28) }
            Children [ @text_small(rank.to_string()) ]
            --
            Node { flex_grow: 1.0 }
            Children [ @text_small(name) ]
            --
            @text_small(score)
        ]
    }
}

// ---------------------------------------------------------------- message log

/// A scrolling list of messages that keeps the newest in view and drops the oldest past `max_lines`.
/// Add messages by triggering [`LogMessage`].
#[derive(Component, Clone, Debug)]
pub struct MessageLog {
    pub max_lines: usize,
}

impl Default for MessageLog {
    fn default() -> Self {
        Self { max_lines: 100 }
    }
}

pub fn message_log(height: f32, max_lines: usize) -> impl Scene {
    let log = MessageLog { max_lines };
    bsn! {
        Node { height: px(height), flex_direction: FlexDirection::Column, row_gap: px(2), overflow: Overflow::scroll_y(), padding: UiRect::all(px(GAP_SMALL)) }
        ScrollArea
        TouchScroll
        ScrollPosition
        log
    }
}

/// Appends a line to a [`MessageLog`].
#[derive(Event, Clone, Debug)]
pub struct LogMessage {
    pub log: Entity,
    pub text: String,
}

fn push_log_message(message: On<LogMessage>, mut logs: Query<(&MessageLog, Option<&Children>, &mut ScrollPosition)>, mut commands: Commands) {
    let Ok((log, children, mut scroll)) = logs.get_mut(message.log) else { return };
    let count = children.map_or(0, |children| children.len());
    if count >= log.max_lines
        && let Some(oldest) = children.and_then(|children| children.first())
    {
        commands.entity(*oldest).despawn();
    }
    commands.queue_spawn_scene(text_small(message.text.clone())).insert(ChildOf(message.log));
    // Past the end is clamped by layout: this keeps the newest line in view.
    scroll.y = f32::MAX;
}

// ---------------------------------------------------------------- HUD and safe area

/// The part of the window a notch, rounded corners or system bars may cover, in UI points. The HUD root
/// keeps clear of it. On the web it comes from the page's `env(safe-area-inset-*)`; natively, set
/// `MUGON_UI_SAFE_AREA="top,right,bottom,left"` to try a layout.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct SafeArea {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[cfg(not(target_arch = "wasm32"))]
fn measure_safe_area() -> SafeArea {
    let Ok(value) = std::env::var("MUGON_UI_SAFE_AREA") else { return SafeArea::default() };
    let parts: Vec<f32> = value.split(',').filter_map(|part| part.trim().parse().ok()).collect();
    match parts[..] {
        [top, right, bottom, left] => SafeArea { top, right, bottom, left },
        _ => SafeArea::default(),
    }
}

/// Reads `env(safe-area-inset-*)` through a throwaway element's computed padding.
#[cfg(target_arch = "wasm32")]
fn measure_safe_area() -> SafeArea {
    use wasm_bindgen::JsCast;
    let measure = || -> Option<SafeArea> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let body = document.body()?;
        let probe: web_sys::HtmlElement = document.create_element("div").ok()?.dyn_into().ok()?;
        probe
            .style()
            .set_property(
                "cssText",
                "position:fixed;visibility:hidden;pointer-events:none;padding:env(safe-area-inset-top,0px) env(safe-area-inset-right,0px) env(safe-area-inset-bottom,0px) env(safe-area-inset-left,0px)",
            )
            .ok()?;
        body.append_child(&probe).ok()?;
        let style = window.get_computed_style(&probe).ok()??;
        let read = |name: &str| style.get_property_value(name).ok().and_then(|value| value.trim_end_matches("px").parse::<f32>().ok()).unwrap_or(0.0);
        let area = SafeArea { top: read("padding-top"), right: read("padding-right"), bottom: read("padding-bottom"), left: read("padding-left") };
        body.remove_child(&probe).ok()?;
        Some(area)
    };
    measure().unwrap_or_default()
}

fn read_safe_area(mut safe_area: ResMut<SafeArea>, device: Res<crate::device::Device>) {
    let measured = measure_safe_area();
    let scaled = SafeArea { top: measured.top / device.zoom, right: measured.right / device.zoom, bottom: measured.bottom / device.zoom, left: measured.left / device.zoom };
    if *safe_area != scaled {
        *safe_area = scaled;
    }
}

/// Where a HUD element sits in the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Component, Default, Clone)]
pub struct HudRoot;

/// A full-window layer for a HUD that lets pointers through. Put [`hud_anchor`]s in it.
pub fn hud_root() -> impl Scene {
    bsn! {
        HudRoot
        Node { position_type: PositionType::Absolute, left: px(0), top: px(0), width: percent(100), height: percent(100) }
        GlobalZIndex(50)
        Pickable::IGNORE
    }
}

/// A region of the HUD root pinned to `anchor`, kept clear of the [`SafeArea`]; children are laid out in a row.
pub fn hud_anchor(anchor: HudAnchor) -> impl Scene {
    let (justify, align) = match anchor {
        HudAnchor::TopLeft => (JustifyContent::FlexStart, AlignItems::FlexStart),
        HudAnchor::Top => (JustifyContent::Center, AlignItems::FlexStart),
        HudAnchor::TopRight => (JustifyContent::FlexEnd, AlignItems::FlexStart),
        HudAnchor::Left => (JustifyContent::FlexStart, AlignItems::Center),
        HudAnchor::Center => (JustifyContent::Center, AlignItems::Center),
        HudAnchor::Right => (JustifyContent::FlexEnd, AlignItems::Center),
        HudAnchor::BottomLeft => (JustifyContent::FlexStart, AlignItems::FlexEnd),
        HudAnchor::Bottom => (JustifyContent::Center, AlignItems::FlexEnd),
        HudAnchor::BottomRight => (JustifyContent::FlexEnd, AlignItems::FlexEnd),
    };
    bsn! {
        HudInsets
        Node { position_type: PositionType::Absolute, left: px(0), right: px(0), top: px(0), bottom: px(0), justify_content: justify, align_items: align }
        Pickable::IGNORE
    }
}

/// Marks a [`hud_anchor`] whose edges follow the [`SafeArea`].
#[derive(Component, Default, Clone)]
struct HudInsets;

fn apply_safe_area(safe_area: Res<SafeArea>, mut anchors: Query<&mut Node, With<HudInsets>>, added: Query<(), Added<HudInsets>>) {
    if !safe_area.is_changed() && added.is_empty() {
        return;
    }
    for mut node in &mut anchors {
        node.left = px(safe_area.left);
        node.right = px(safe_area.right);
        node.top = px(safe_area.top);
        node.bottom = px(safe_area.bottom);
    }
}

pub mod prelude {
    pub use super::{GameUiPlugin, HudAnchor, HudRoot, LogMessage, MessageLog, ProgressBar, SafeArea, Spinner, hud_anchor, hud_root, message_log, player_badge, progress_bar, scoreboard_row, spinner};
}
