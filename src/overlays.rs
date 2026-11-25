//! Overlays: dialog, tooltip, popover and toast, themed like the rest of the crate.
//!
//! - [`dialog`]: a modal dialog with a title bar, body and action row; it closes itself on the close
//!   button, Escape or a click outside, and `RequestClose` triggered from any of its buttons.
//! - [`Tooltip`]: put on any entity; its text shows next to the pointer after a short hover.
//! - [`popover`]: a panel that positions itself next to the entity it is spawned under.
//! - [`Toast`]: trigger it to show a short message that fades out by itself.

use bevy::feathers::controls::{FeathersDialog, FeathersDialogBody, FeathersDialogClose, FeathersDialogFooter, FeathersDialogHeader};
use bevy::picking::Pickable;
use bevy::picking::pointer::PointerLocation;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{GlobalZIndex, UiScale};
use bevy::ui_widgets::RequestClose;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

use crate::widgets::{panel, text_small};

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TooltipState>().add_observer(show_toast).add_systems(Update, (show_tooltips, expire_toasts));
    }
}

/// A modal dialog. `body` and `actions` (usually buttons) are the contents of its two sections.
pub fn dialog(title: String, width: f32, body: Box<dyn SceneList>, actions: Box<dyn SceneList>) -> impl Scene {
    let contents: Box<dyn SceneList> = Box::new(bsn_list! {
        @FeathersDialogHeader
        Children [ Text(title) bevy::feathers::theme::ThemedText -- @FeathersDialogClose ]
        --
        @FeathersDialogBody
        Children [ {body} ]
        --
        @FeathersDialogFooter
        Children [ {actions} ]
    });
    bsn! {
        @FeathersDialog { @contents: {contents}, @width: px(width) }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
        on(|close: On<RequestClose>, mut commands: Commands| {
            commands.entity(close.event_target()).despawn();
        })
    }
}

/// A panel that positions itself on `side` of its parent (flipping to the other side when there is no
/// room). Spawn it as a child of the entity it belongs to.
pub fn popover(side: PopoverSide, content: Box<dyn SceneList>) -> impl Scene {
    let positions = vec![PopoverPlacement { side, align: PopoverAlign::Start, gap: 4.0 }, PopoverPlacement { side: side.mirror(), align: PopoverAlign::Start, gap: 4.0 }];
    bsn! {
        @panel()
        Node { position_type: PositionType::Absolute }
        Popover { positions: {positions}, window_margin: 8.0 }
        GlobalZIndex(110)
        Children [ {content} ]
    }
}

/// Text shown in a bubble next to the pointer while the entity is hovered for a moment.
#[derive(Component, Clone, Debug, Default)]
#[require(Hovered)]
pub struct Tooltip(pub String);

const TOOLTIP_DELAY: f64 = 0.5;

#[derive(Resource, Default)]
struct TooltipState {
    target: Option<Entity>,
    since: f64,
    bubble: Option<Entity>,
}

#[derive(Component, Default, Clone)]
struct TooltipBubble;

fn tooltip_bubble(content: String) -> impl Scene {
    bsn! {
        @panel()
        Node { position_type: PositionType::Absolute, padding: UiRect::axes(px(8), px(4)) }
        GlobalZIndex(150)
        Pickable::IGNORE
        TooltipBubble
        Children [ @text_small(content) ]
    }
}

fn show_tooltips(
    mut state: ResMut<TooltipState>,
    time: Res<Time>,
    ui_scale: Res<UiScale>,
    pointers: Query<&PointerLocation>,
    targets: Query<(Entity, &Tooltip, &Hovered)>,
    mut bubbles: Query<&mut Node, With<TooltipBubble>>,
    mut commands: Commands,
) {
    let hovered = targets.iter().find(|(_, _, hovered)| hovered.0).map(|(entity, tooltip, _)| (entity, tooltip.0.clone()));
    let hovered_entity = hovered.as_ref().map(|(entity, _)| *entity);
    if state.target != hovered_entity {
        if let Some(bubble) = state.bubble.take() {
            commands.entity(bubble).despawn();
        }
        state.target = hovered_entity;
        state.since = time.elapsed_secs_f64();
    }
    if let Some((_, content)) = hovered
        && state.bubble.is_none()
        && time.elapsed_secs_f64() - state.since >= TOOLTIP_DELAY
        && !content.is_empty()
    {
        state.bubble = Some(commands.queue_spawn_scene(tooltip_bubble(content)).id());
    }
    if let Some(bubble) = state.bubble
        && let Ok(mut node) = bubbles.get_mut(bubble)
        && let Some(cursor) = pointers.iter().find_map(|pointer| pointer.location.as_ref().map(|location| location.position))
    {
        node.left = px((cursor.x + 12.0) / ui_scale.0);
        node.top = px((cursor.y + 18.0) / ui_scale.0);
    }
}

/// Shows `text` for `seconds` in a stack at the bottom of the window.
#[derive(Event, Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub seconds: f32,
}

impl Toast {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), seconds: 3.0 }
    }
}

#[derive(Component, Default, Clone)]
struct ToastLayer;

#[derive(Component, Clone)]
struct ToastTimer(Timer);

impl Default for ToastTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Once))
    }
}

fn toast_scene(text: String, seconds: f32) -> impl Scene {
    let timer = ToastTimer(Timer::from_seconds(seconds, TimerMode::Once));
    bsn! {
        @panel()
        Node { padding: UiRect::axes(px(12), px(8)) }
        timer
        Pickable::IGNORE
        Children [ @text_small(text) ]
    }
}

fn show_toast(toast: On<Toast>, layers: Query<Entity, With<ToastLayer>>, mut commands: Commands) {
    let layer = layers.single().unwrap_or_else(|_| {
        commands
            .queue_spawn_scene(bsn! {
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    bottom: px(24),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(8),
                }
                GlobalZIndex(120)
                Pickable::IGNORE
                ToastLayer
            })
            .id()
    });
    commands.queue_spawn_scene(toast_scene(toast.text.clone(), toast.seconds)).insert(ChildOf(layer));
}

fn expire_toasts(mut toasts: Query<(Entity, &mut ToastTimer)>, time: Res<Time>, mut commands: Commands) {
    for (entity, mut timer) in &mut toasts {
        if timer.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub mod prelude {
    pub use super::{OverlaysPlugin, Toast, Tooltip, dialog, popover};
}
