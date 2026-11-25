//! The look of the platform's widgets: small scenes built on feathers, all drawing from the same
//! theme tokens. Nothing here knows what a widget does: games and tools patch their own components
//! and observers onto the returned scenes (`bsn! { @tile("Short".into(), TileSize::Normal) MyMarker on(my_observer) }`).

use bevy::feathers::constants::{fonts, size};
use bevy::feathers::controls::FeathersSlider;
use bevy::feathers::focus::FocusIndicator;
use bevy::feathers::font_styles::InheritableFont;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemeBackgroundColor, ThemeBorderColor, ThemeTextColor, ThemeToken, ThemedText};
use bevy::feathers::tokens;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::cursor::EntityCursor;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSourceTemplate, FontWeight};
use bevy::ui::{ComputedNode, InteractionDisabled, Pressed, ScrollPosition, Selected, UiGlobalTransform, UiScale};
use bevy::ui_widgets::{Button as ActivateButton, SliderPrecision};
use bevy::window::SystemCursorIcon;

use crate::theme::{GAP, GAP_SMALL, RADIUS, palette, token};

/// How [`style_choices`] colours a choice.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChoiceKind {
    #[default]
    Tile,
    Nav,
    Swatch,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileSize {
    Normal,
    Compact,
    /// One option per line.
    List,
}

/// Spawns `scene` as a child of `parent`.
pub fn spawn_in(commands: &mut Commands, parent: Entity, scene: impl Scene) -> Entity {
    commands.queue_spawn_scene(scene).insert(ChildOf(parent)).id()
}

/// Text in the theme's font.
pub fn text(content: String, font: &'static str, font_size: FontSize, color: ThemeToken) -> impl Scene {
    bsn! {
        Text(content)
        TextFont {
            font: FontSourceTemplate::Handle(font),
            font_size: font_size,
            weight: FontWeight::NORMAL,
        }
        bevy::app::PropagateOver<TextFont>
        ThemeTextColor(color)
    }
}

pub fn title(content: String) -> impl Scene {
    text(content, fonts::BOLD, FontSize::Px(16.0), tokens::TEXT_MAIN)
}

pub fn heading(content: String) -> impl Scene {
    bsn! {
        @text(content, fonts::BOLD, FontSize::Px(13.0), tokens::TEXT_DIM)
        Node { margin: UiRect::top(px(GAP)) }
    }
}

pub fn hint(content: String) -> impl Scene {
    text(content, fonts::REGULAR, size::SMALL_FONT, tokens::TEXT_DIM)
}

pub fn text_small(content: String) -> impl Scene {
    text(content, fonts::REGULAR, size::COMPACT_FONT, tokens::TEXT_MAIN)
}

pub fn field_label(content: &'static str) -> impl Scene {
    bsn! {
        Node { width: px(92), flex_shrink: 0.0 }
        Children [ @text(content.to_string(), fonts::REGULAR, size::COMPACT_FONT, tokens::TEXT_MAIN) ]
    }
}

/// A floating surface.
pub fn panel() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            padding: px(GAP),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(RADIUS)),
        }
        ThemeBackgroundColor(token::PANEL_BG)
        ThemeBorderColor(token::PANEL_BORDER)
    }
}

pub fn divider() -> impl Scene {
    bsn! {
        Node { height: px(1), flex_shrink: 0.0 }
        ThemeBackgroundColor(token::DIVIDER)
    }
}

pub fn divider_vertical() -> impl Scene {
    bsn! { Node { width: px(1), margin: UiRect::vertical(px(2)) } ThemeBackgroundColor(token::DIVIDER) }
}

pub fn column(gap: f32) -> impl Scene {
    bsn! { Node { flex_direction: FlexDirection::Column, row_gap: px(gap) } }
}

pub fn wrap(gap: f32) -> impl Scene {
    bsn! { Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(gap), row_gap: px(gap) } }
}

/// The themed fill of a choice; swatches have their own colour instead.
fn surface() -> impl Scene {
    bsn! { ThemeBackgroundColor(token::CHOICE_BG) }
}

/// What every pressable choice shares; [`style_choices`] colours it. Emits `Activate`.
pub fn choice(kind: ChoiceKind) -> impl Scene {
    bsn! {
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(RADIUS)),
        }
        ActivateButton
        Hovered
        kind
        EntityCursor::System(SystemCursorIcon::Pointer)
        TabIndex(0)
        FocusIndicator
        ThemeBorderColor(tokens::BUTTON_PLAIN_BG)
        InheritableThemeTextColor(token::CHOICE_TEXT)
        InheritableFont {
            font: fonts::REGULAR,
            font_size: size::COMPACT_FONT,
            weight: FontWeight::NORMAL,
        }
    }
}

pub fn tile(label: String, tile_size: TileSize) -> impl Scene {
    let (basis, grow): (Val, f32) = match tile_size {
        TileSize::Normal => (px(86), 1.0),
        TileSize::Compact => (px(72), 1.0),
        TileSize::List => (percent(100), 1.0),
    };
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node {
            min_height: px(36),
            flex_basis: basis,
            flex_grow: grow,
            padding: UiRect::horizontal(px(GAP)),
        }
        Children [ Text(label) ThemedText TextLayout { justify: Justify::Center } ]
    }
}

/// A tile that holds a state: use with `Selected`.
pub fn toggle_tile(label: &'static str) -> impl Scene {
    let label = label.to_string();
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node {
            min_height: px(36),
            flex_basis: px(86),
            flex_grow: 1.0,
            padding: UiRect::horizontal(px(GAP)),
        }
        Children [ Text(label) ThemedText ]
    }
}

/// A tile showing one cell (`cell`, in texels) of a strip of pictures.
pub fn picture_tile(image: Handle<Image>, cell: Rect) -> impl Scene {
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node { width: px(52), height: px(52), flex_shrink: 0.0 }
        Children [
            Node { width: px(40), height: px(40) }
            ImageNode { image: image, rect: {Some(cell)} }
        ]
    }
}

/// A colour to pick; the fill is the data, only the ring is themed.
pub fn swatch(color: [u8; 3]) -> impl Scene {
    let fill = Color::srgb_u8(color[0], color[1], color[2]);
    bsn! {
        @choice(ChoiceKind::Swatch)
        Node { width: px(28), height: px(28), border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(6)) }
        BackgroundColor(fill)
    }
}

/// A small text button next to swatches.
pub fn chip(label: &'static str) -> impl Scene {
    let label = label.to_string();
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node { height: px(28), padding: UiRect::horizontal(px(GAP)), border_radius: BorderRadius::all(px(6)) }
        Children [ Text(label) ThemedText ]
    }
}

/// Set `DisabledFlag(true)` to disable a widget, `false` to enable it ([`apply_disabled`]).
#[derive(Component, Default, Clone, Copy)]
pub struct DisabledFlag(pub bool);

pub fn button(label: String, enabled: bool) -> impl Scene {
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node { min_height: px(32), flex_grow: 1.0, padding: UiRect::horizontal(px(GAP)) }
        DisabledFlag({!enabled})
        Children [ Text(label) ThemedText ]
    }
}

/// An entry of a navigation rail or tab strip: use with `Selected`.
pub fn nav_button(label: &'static str) -> impl Scene {
    let label = label.to_string();
    bsn! {
        @choice(ChoiceKind::Nav)
        @surface()
        Node { min_height: px(40), flex_basis: px(0), flex_grow: 1.0, padding: UiRect::horizontal(px(GAP_SMALL)) }
        Children [ Text(label) ThemedText ]
    }
}

/// An icon-only button: a square with the icon centred.
pub fn icon_button(icon: Handle<Image>) -> impl Scene {
    bsn! {
        @choice(ChoiceKind::Tile)
        @surface()
        Node { width: px(36), height: px(32) }
        Children [
            Node { width: px(20), height: px(20) }
            ImageNode { image: icon, color: palette::DARK_0 }
        ]
    }
}

/// A labelled 0..=1 slider. `control` is patched onto the slider entity: its binding.
pub fn slider(label: &'static str, ends: Option<(&'static str, &'static str)>, control: impl Scene) -> impl Scene {
    bsn! {
        Node { flex_direction: FlexDirection::Column, row_gap: px(2) }
        Children [
            Node { align_items: AlignItems::Center, column_gap: px(GAP_SMALL) }
            Children [
                @field_label(label)
                --
                @FeathersSlider
                SliderPrecision(2)
                @{control}
            ]
            --
            @slider_ends(ends)
        ]
    }
}

fn slider_ends(ends: Option<(&'static str, &'static str)>) -> impl Scene {
    let (low, high) = ends.unwrap_or(("", ""));
    bsn! {
        Node {
            display: {if ends.is_some() { Display::Flex } else { Display::None }},
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::left(px(92 + GAP_SMALL as i32)),
        }
        Children [
            @text(low.to_string(), fonts::REGULAR, size::EXTRA_SMALL_FONT, tokens::TEXT_DIM)
            --
            @text(high.to_string(), fonts::REGULAR, size::EXTRA_SMALL_FONT, tokens::TEXT_DIM)
        ]
    }
}

/// A labelled slider for one 0..=255 colour channel. `control` is patched onto the slider entity.
pub fn channel_slider(label: &'static str, control: impl Scene) -> impl Scene {
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(GAP_SMALL) }
        Children [
            @field_label(label)
            --
            @FeathersSlider { @max: 255.0 }
            SliderPrecision(0)
            @{control}
        ]
    }
}

/// Marks a scrolling area that follows a finger dragging on it ([`touch_scroll`]); the wheel is
/// handled by feathers' `ScrollArea`.
#[derive(Component, Default, Clone)]
pub struct TouchScroll;

/// A node's rectangle in window points.
pub fn window_rect(node: &ComputedNode, transform: &UiGlobalTransform, window: &Window) -> Rect {
    let scale = window.scale_factor();
    let center = transform.translation / scale;
    let half = node.size() / scale * 0.5;
    Rect::from_corners(center - half, center + half)
}

pub(crate) fn touch_scroll(
    touches: Res<Touches>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
    areas: Query<(&ComputedNode, &UiGlobalTransform), With<TouchScroll>>,
    mut positions: Query<&mut ScrollPosition, With<TouchScroll>>,
    mut tracked: Local<Option<(u64, f32)>>,
) {
    let Ok(window) = windows.single() else { return };
    for touch in touches.iter_just_pressed() {
        let inside = areas.iter().any(|(node, transform)| window_rect(node, transform, window).contains(touch.position()));
        if inside {
            *tracked = Some((touch.id(), touch.position().y));
        }
    }
    let Some((id, last)) = *tracked else { return };
    let Some(touch) = touches.get_pressed(id) else {
        *tracked = None;
        return;
    };
    let delta = (touch.position().y - last) / ui_scale.0;
    *tracked = Some((id, touch.position().y));
    for mut position in &mut positions {
        position.y = (position.y - delta).max(0.0);
    }
}

/// Mirrors [`DisabledFlag`] onto [`InteractionDisabled`].
pub(crate) fn apply_disabled(flags: Query<(Entity, &DisabledFlag, Has<InteractionDisabled>), Changed<DisabledFlag>>, mut commands: Commands) {
    for (entity, flag, disabled) in &flags {
        if flag.0 && !disabled {
            commands.entity(entity).insert(InteractionDisabled);
        } else if !flag.0 && disabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
    }
}

type ChoiceState<'a> = (
    Entity,
    &'a ChoiceKind,
    &'a Hovered,
    Has<Pressed>,
    Has<Selected>,
    Has<InteractionDisabled>,
    Option<&'a ThemeBackgroundColor>,
    &'a InheritableThemeTextColor,
    &'a ThemeBorderColor,
);

/// Colours choices by hover / press / selected / disabled, from the theme tokens.
pub(crate) fn style_choices(choices: Query<ChoiceState>, mut commands: Commands) {
    for (entity, kind, hovered, pressed, selected, disabled, background, text, border) in &choices {
        let (background_token, text_token, border_token) = match (kind, disabled, pressed, selected, hovered.0) {
            (_, true, ..) => (token::CHOICE_BG_DISABLED, token::CHOICE_TEXT_DISABLED, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Swatch, false, _, true, _) => (token::CHOICE_BG, token::CHOICE_TEXT, token::SWATCH_RING),
            (ChoiceKind::Swatch, false, _, false, true) => (token::CHOICE_BG, token::CHOICE_TEXT, token::CHOICE_BORDER_SELECTED),
            (ChoiceKind::Swatch, ..) => (token::CHOICE_BG, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Nav, false, _, true, _) => (token::CHOICE_BG_SELECTED, token::CHOICE_TEXT_SELECTED, token::CHOICE_BORDER_SELECTED),
            (ChoiceKind::Nav, false, true, ..) => (token::CHOICE_BG_PRESSED, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Nav, false, _, _, true) => (token::NAV_BG_HOVER, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Nav, ..) => (tokens::BUTTON_PLAIN_BG, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Tile, false, true, ..) => (token::CHOICE_BG_PRESSED, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Tile, false, _, true, _) => (token::CHOICE_BG_SELECTED, token::CHOICE_TEXT_SELECTED, token::CHOICE_BORDER_SELECTED),
            (ChoiceKind::Tile, false, _, _, true) => (token::CHOICE_BG_HOVER, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
            (ChoiceKind::Tile, ..) => (token::CHOICE_BG, token::CHOICE_TEXT, tokens::BUTTON_PLAIN_BG),
        };
        let mut entity_commands = commands.entity(entity);
        if background.is_some_and(|background| background.0 != background_token) {
            entity_commands.insert(ThemeBackgroundColor(background_token));
        }
        if text.0 != text_token {
            entity_commands.insert(InheritableThemeTextColor(text_token));
        }
        if border.0 != border_token {
            entity_commands.insert(ThemeBorderColor(border_token));
        }
    }
}
