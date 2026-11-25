//! Widgets for tools with panels: menu bar, splitter, tree and virtualized grid.

use std::sync::Arc;

use bevy::feathers::constants::{fonts, size};
use bevy::feathers::controls::{FeathersDisclosureToggle, FeathersMenu, FeathersMenuButton, FeathersMenuDivider, FeathersMenuItem, FeathersMenuPopup};
use bevy::feathers::font_styles::InheritableFont;
use bevy::feathers::theme::ThemedText;
use bevy::picking::cursor::EntityCursor;
use bevy::picking::events::PointerDrag;
use bevy::prelude::*;
use bevy::ui::{Checked, ComputedNode, ScrollPosition, UiScale};
use bevy::ui_widgets::{ScrollArea, ValueChange, checkbox_self_update};
use bevy::window::SystemCursorIcon;

use crate::theme::{GAP_SMALL, token};
use crate::widgets::{ChoiceKind, TouchScroll, choice, text_small};

pub struct ToolUiPlugin;

impl Plugin for ToolUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(drag_splitter).add_systems(Update, update_virtual_grids);
    }
}

// ---------------------------------------------------------------- menus

/// A row of menus along the top of a window.
pub fn menu_bar() -> impl Scene {
    bsn! { Node { align_items: AlignItems::Center, column_gap: px(2), padding: UiRect::axes(px(GAP_SMALL), px(2)) } }
}

/// A drop-down menu for a [`menu_bar`]: a button with `label` that opens `items` ([`menu_item`]s and
/// [`menu_divider`]s). Menu items emit `Activate`.
pub fn menu(label: &'static str, items: Box<dyn SceneList>) -> impl Scene {
    let text = label.to_string();
    let caption: Box<dyn SceneList> = Box::new(bsn_list! { Text(text) ThemedText });
    bsn! {
        @FeathersMenu
        Children [
            @FeathersMenuButton { @caption: {caption} }
            --
            @FeathersMenuPopup
            Children [ {items} ]
        ]
    }
}

/// An entry of a [`menu`], with the shortcut shown dimmed on the right. Add `InteractionDisabled` to grey it out.
pub fn menu_item(label: String, shortcut: Option<&'static str>) -> impl Scene {
    let caption: Box<dyn SceneList> = Box::new(bsn_list! {
        Text(label) ThemedText
        --
        Node { flex_grow: 1.0, min_width: px(24) }
        --
        {shortcut.map(|shortcut| bsn_list! { @text_small(shortcut.to_string()) })}
    });
    bsn! { @FeathersMenuItem { @caption: {caption} } }
}

pub fn menu_divider() -> impl Scene {
    bsn! { @FeathersMenuDivider }
}

// ---------------------------------------------------------------- splitter

/// Which neighbour a [`Splitter`] resizes.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum Resizes {
    /// The sibling before the splitter (left or above it).
    #[default]
    Before,
    /// The sibling after the splitter (right or below it).
    After,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum SplitAxis {
    /// Splits left from right: dragging changes widths.
    #[default]
    Horizontal,
    /// Splits top from bottom: dragging changes heights.
    Vertical,
}

/// A draggable divider that resizes one of its siblings, between `min` and `max` points.
#[derive(Component, Clone, Default, Debug)]
pub struct Splitter {
    pub axis: SplitAxis,
    pub resizes: Resizes,
    pub min: f32,
    pub max: f32,
}

pub fn splitter(axis: SplitAxis, resizes: Resizes, min: f32, max: f32) -> impl Scene {
    let state = Splitter { axis, resizes, min, max };
    let (width, height, cursor) = match axis {
        SplitAxis::Horizontal => (px(6), percent(100), SystemCursorIcon::ColResize),
        SplitAxis::Vertical => (percent(100), px(6), SystemCursorIcon::RowResize),
    };
    bsn! {
        Node { width: width, height: height, flex_shrink: 0.0 }
        state
        EntityCursor::System(cursor)
    }
}

fn drag_splitter(
    drag: On<PointerDrag>,
    splitters: Query<(&Splitter, &ChildOf)>,
    children: Query<&Children>,
    sizes: Query<&ComputedNode>,
    mut nodes: Query<&mut Node>,
    ui_scale: Res<UiScale>,
) {
    let Ok((splitter, parent)) = splitters.get(drag.entity) else { return };
    let Ok(siblings) = children.get(parent.parent()) else { return };
    let Some(index) = siblings.iter().position(|sibling| sibling == drag.entity) else { return };
    let target = match splitter.resizes {
        Resizes::Before => index.checked_sub(1).and_then(|before| siblings.get(before)).copied(),
        Resizes::After => siblings.get(index + 1).copied(),
    };
    let Some(target) = target else { return };
    let (Ok(computed), Ok(mut node)) = (sizes.get(target), nodes.get_mut(target)) else { return };
    let delta = match splitter.axis {
        SplitAxis::Horizontal => drag.delta.x,
        SplitAxis::Vertical => drag.delta.y,
    } / ui_scale.0
        * if splitter.resizes == Resizes::Before { 1.0 } else { -1.0 };
    let current = match splitter.axis {
        SplitAxis::Horizontal => computed.size().x,
        SplitAxis::Vertical => computed.size().y,
    } * computed.inverse_scale_factor();
    let wanted = (current + delta).clamp(splitter.min, splitter.max.max(splitter.min));
    match splitter.axis {
        SplitAxis::Horizontal => node.width = px(wanted),
        SplitAxis::Vertical => node.height = px(wanted),
    }
}

// ---------------------------------------------------------------- tree

/// The label button of a tree row: observe its `Activate` to select, and add `Selected` to show it.
#[derive(Component, Default, Clone)]
pub struct TreeLabel;

#[derive(Component, Default, Clone)]
struct TreeBody;

fn tree_label(label: String) -> impl Scene {
    bsn! {
        @choice(ChoiceKind::Nav)
        TreeLabel
        Node { min_height: px(22), flex_grow: 1.0, justify_content: JustifyContent::FlexStart, padding: UiRect::horizontal(px(GAP_SMALL)) }
        bevy::feathers::theme::ThemeBackgroundColor(token::CHOICE_BG)
        InheritableFont { font: fonts::REGULAR, font_size: size::COMPACT_FONT, weight: bevy::text::FontWeight::NORMAL }
        Children [ Text(label) ThemedText ]
    }
}

/// A row that cannot be expanded.
pub fn tree_leaf(label: String) -> impl Scene {
    bsn! {
        Node { align_items: AlignItems::Center, column_gap: px(GAP_SMALL), padding: UiRect::left(px(12 + GAP_SMALL as i32)) }
        Children [ @tree_label(label) ]
    }
}

/// A row with children that can be folded away. `children` are [`tree_node`]s and [`tree_leaf`]s.
pub fn tree_node(label: String, expanded: bool, children: Box<dyn SceneList>) -> impl Scene {
    let display = if expanded { Display::Flex } else { Display::None };
    let toggle: Box<dyn SceneList> = if expanded {
        Box::new(bsn_list! { @FeathersDisclosureToggle Checked on(checkbox_self_update) on(toggle_tree_body) })
    } else {
        Box::new(bsn_list! { @FeathersDisclosureToggle on(checkbox_self_update) on(toggle_tree_body) })
    };
    bsn! {
        Node { flex_direction: FlexDirection::Column }
        Children [
            Node { align_items: AlignItems::Center, column_gap: px(GAP_SMALL) }
            Children [
                {toggle}
                --
                @tree_label(label)
            ]
            --
            TreeBody
            Node { flex_direction: FlexDirection::Column, padding: UiRect::left(px(14)), display: display }
            Children [ {children} ]
        ]
    }
}

fn toggle_tree_body(change: On<ValueChange<bool>>, parents: Query<&ChildOf>, children: Query<&Children>, mut bodies: Query<&mut Node, With<TreeBody>>) {
    let Ok(header) = parents.get(change.source) else { return };
    let Ok(node) = parents.get(header.parent()) else { return };
    let Ok(siblings) = children.get(node.parent()) else { return };
    for &sibling in siblings {
        if let Ok(mut body) = bodies.get_mut(sibling) {
            body.display = if change.value { Display::Flex } else { Display::None };
        }
    }
}

// ---------------------------------------------------------------- virtual grid

/// Makes the scene of item `index`.
pub type MakeItem = Arc<dyn Fn(usize) -> Box<dyn SceneList> + Send + Sync>;

/// A scrolling grid of `count` equally sized items of which only the visible ones exist as entities, for
/// thousands of items. Spawn [`virtual_grid`] and insert this on it; change `count` (or anything else) to rebuild.
#[derive(Component, Clone)]
pub struct VirtualGrid {
    pub count: usize,
    pub cell: Vec2,
    pub gap: f32,
    pub make: MakeItem,
}

/// An item of a [`VirtualGrid`] (the wrapper entity that holds its scene).
#[derive(Component, Clone, Copy, Debug)]
pub struct VirtualItem(pub usize);

#[derive(Component, Default, Clone)]
struct VirtualGridContent;

/// The scrolling container of a [`VirtualGrid`], `height` points tall.
pub fn virtual_grid(height: f32) -> impl Scene {
    bsn! {
        Node { height: px(height), flex_shrink: 0.0, overflow: Overflow::scroll_y() }
        ScrollArea
        TouchScroll
        ScrollPosition
        Children [ VirtualGridContent Node { width: percent(100) } ]
    }
}

/// What a grid showed last time: when it matches, nothing needs doing.
#[derive(Component, Default)]
struct GridShown {
    first: usize,
    end: usize,
    columns: usize,
}

/// Rows kept above and below the visible ones, so scrolling does not show gaps.
const OVERSCAN_ROWS: usize = 1;

type GridQuery = (Entity, Ref<'static, VirtualGrid>, &'static ComputedNode, &'static ScrollPosition, &'static Children, Option<&'static mut GridShown>);

fn update_virtual_grids(
    mut grids: Query<GridQuery>,
    mut contents: Query<&mut Node, With<VirtualGridContent>>,
    items: Query<(Entity, &VirtualItem, &ChildOf)>,
    mut commands: Commands,
) {
    for (grid_entity, grid, computed, scroll, children, shown) in &mut grids {
        let Some(content) = children.iter().find(|child| contents.contains(*child)) else { continue };
        let width = computed.size().x * computed.inverse_scale_factor();
        let height = computed.size().y * computed.inverse_scale_factor();
        if width <= 0.0 || height <= 0.0 {
            continue;
        }
        let step = grid.cell + Vec2::splat(grid.gap);
        let columns = (((width + grid.gap) / step.x).floor() as usize).max(1);
        let rows = grid.count.div_ceil(columns);
        let first_row = ((scroll.y / step.y).floor() as usize).saturating_sub(OVERSCAN_ROWS);
        let end_row = (((scroll.y + height) / step.y).ceil() as usize + OVERSCAN_ROWS).min(rows);
        let (first, end) = (first_row * columns, (end_row * columns).min(grid.count));

        let current = GridShown { first, end, columns };
        let unchanged = shown.as_ref().is_some_and(|shown| shown.first == first && shown.end == end && shown.columns == columns);
        if unchanged && !grid.is_changed() {
            continue;
        }
        let rebuild_all = grid.is_changed() || shown.as_ref().is_none_or(|shown| shown.columns != columns);
        if let Ok(mut node) = contents.get_mut(content) {
            node.height = px((rows as f32 * step.y - grid.gap).max(0.0));
        }
        let mut present = std::collections::HashSet::new();
        for (entity, item, parent) in &items {
            if parent.parent() != content {
                continue;
            }
            if rebuild_all || item.0 < first || item.0 >= end {
                commands.entity(entity).despawn();
            } else {
                present.insert(item.0);
            }
        }
        for index in first..end {
            if present.contains(&index) {
                continue;
            }
            let (column, row) = (index % columns, index / columns);
            let wrapper = commands
                .spawn((
                    VirtualItem(index),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(column as f32 * step.x),
                        top: px(row as f32 * step.y),
                        width: px(grid.cell.x),
                        height: px(grid.cell.y),
                        ..default()
                    },
                    ChildOf(content),
                ))
                .id();
            commands.entity(wrapper).queue_spawn_related_scenes::<Children>((grid.make)(index));
        }
        match shown {
            Some(mut shown) => *shown = current,
            None => {
                commands.entity(grid_entity).insert(current);
            }
        }
    }
}

pub mod prelude {
    pub use super::{MakeItem, Resizes, SplitAxis, Splitter, ToolUiPlugin, TreeLabel, VirtualGrid, VirtualItem, menu, menu_bar, menu_divider, menu_item, splitter, tree_leaf, tree_node, virtual_grid};
}
