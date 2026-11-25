//! Inputs: checkbox, dropdown, text field, number input and tabs, themed like the rest of the crate.
//! Feathers provides the behaviour; these wrap it with the platform's look and sensible defaults.
//!
//! - checkbox: toggles itself and emits `ValueChange<bool>` (or read the entity's `Checked`).
//! - [`select`]: emits `ValueChange<Entity>`; the chosen row carries `OptionIndex`.
//! - [`text_field`]: read the text with `EditableText::value()`; it has an optional placeholder.
//! - [`number_input`]: emits `ValueChange<f32>`; insert a new `NumberInputValue` to change it.
//! - [`tabs`]: emits `ValueChange<Option<Entity>>` and keeps the selection itself.

use bevy::feathers::controls::{FeathersCheckbox, FeathersNumberInput, FeathersSelect, FeathersTextInput, FeathersTextInputContainer, list_rows_from_strings};
use bevy::feathers::theme::ThemedText;
use bevy::feathers::tokens;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::cursor::EntityCursor;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui_widgets::{SelectedTab, Tab, TabList, checkbox_self_update, tablist_self_update};
use bevy::window::SystemCursorIcon;

use crate::theme::{GAP_SMALL, RADIUS, token};
use crate::widgets::{ChoiceKind, text};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (select_first_tab, update_placeholders).in_set(crate::MugonUiSystems::Sync));
    }
}

pub fn checkbox(label: &'static str) -> impl Scene {
    let label = label.to_string();
    bsn! { @FeathersCheckbox { @caption: bsn! { Text(label) ThemedText } } on(checkbox_self_update) }
}

/// A dropdown of text options; `selected` is the one shown at first.
pub fn select(options: &[&str], selected: Option<usize>) -> impl Scene {
    let rows = list_rows_from_strings(options.iter().copied(), selected);
    bsn! { @FeathersSelect { @options: {rows} } }
}

/// A single-line text field. `placeholder` is shown, dimmed, while it is empty.
pub fn text_field(visible_width: Option<f32>, placeholder: Option<&'static str>) -> impl Scene {
    let placeholder = placeholder.unwrap_or_default().to_string();
    bsn! {
        @FeathersTextInputContainer
        Node { flex_grow: 0.0, width: percent(100) }
        Children [
            @FeathersTextInput { @visible_width: {visible_width} }
            --
            @text(placeholder, bevy::feathers::constants::fonts::REGULAR, bevy::feathers::constants::size::COMPACT_FONT, tokens::TEXT_DIM)
            Placeholder
            Node { position_type: PositionType::Absolute, left: px(GAP_SMALL + 3.0) }
            Pickable::IGNORE
        ]
    }
}

/// Marks the dim text of a [`text_field`] that shows while the field is empty.
#[derive(Component, Default, Clone)]
pub struct Placeholder;

/// A number input with typing and drag-to-scrub. Patch `SoftLimit`, `HardLimit`, `NumberInputPrecision`,
/// `NumberInputStep` or `NumberInputUnits` onto it to configure it.
pub fn number_input(value: f32) -> impl Scene {
    bsn! { @FeathersNumberInput bevy::feathers::controls::NumberInputValue::F32(value) Node { flex_grow: 0.0, width: percent(100) } }
}

/// A row of tabs; `segmented` draws them as a segmented control (tiles) instead of a tab strip (rail
/// entries). The first tab starts selected.
pub fn tabs(labels: &[&str], segmented: bool) -> impl Scene {
    let kind = if segmented { ChoiceKind::Tile } else { ChoiceKind::Nav };
    let headers: Vec<_> = labels.iter().map(|label| tab_header(label.to_string(), kind)).collect();
    bsn! {
        Node { column_gap: px(2) }
        TabList
        on(tablist_self_update)
        Children [ {headers} ]
    }
}

/// A tab header: a choice without `Activate`, so clicks reach the `TabList`.
fn tab_header(label: String, kind: ChoiceKind) -> impl Scene {
    bsn! {
        Node {
            min_height: px(32),
            flex_basis: px(0),
            flex_grow: 1.0,
            padding: UiRect::horizontal(px(GAP_SMALL * 2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(RADIUS)),
        }
        Tab
        Hovered
        kind
        EntityCursor::System(SystemCursorIcon::Pointer)
        TabIndex(-1)
        bevy::feathers::focus::FocusIndicator
        bevy::feathers::theme::ThemeBackgroundColor(token::CHOICE_BG)
        bevy::feathers::theme::ThemeBorderColor(tokens::BUTTON_PLAIN_BG)
        bevy::feathers::theme::InheritableThemeTextColor(token::CHOICE_TEXT)
        bevy::feathers::font_styles::InheritableFont {
            font: bevy::feathers::constants::fonts::REGULAR,
            font_size: bevy::feathers::constants::size::COMPACT_FONT,
            weight: bevy::text::FontWeight::NORMAL,
        }
        Children [ Text(label) ThemedText ]
    }
}

fn select_first_tab(lists: Query<(Entity, &SelectedTab, &Children), With<TabList>>, tabs: Query<(), With<Tab>>, mut commands: Commands) {
    for (list, selected, children) in &lists {
        if selected.0.is_none()
            && let Some(first) = children.iter().find(|child| tabs.contains(*child))
        {
            commands.entity(list).insert(SelectedTab(Some(first)));
        }
    }
}

/// Shows a text field's placeholder only while the field is empty.
fn update_placeholders(
    fields: Query<(&EditableText, &ChildOf), With<FeathersTextInput>>,
    children: Query<&Children>,
    mut placeholders: Query<&mut Node, With<Placeholder>>,
) {
    for (field, parent) in &fields {
        let Ok(siblings) = children.get(parent.parent()) else { continue };
        for &sibling in siblings {
            if let Ok(mut node) = placeholders.get_mut(sibling) {
                let wanted = if field.value().to_string().is_empty() { Display::Flex } else { Display::None };
                if node.display != wanted {
                    node.display = wanted;
                }
            }
        }
    }
}

pub mod prelude {
    pub use super::{InputsPlugin, Placeholder, checkbox, number_input, select, tabs, text_field};
}
