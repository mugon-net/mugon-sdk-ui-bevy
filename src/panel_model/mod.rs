//! Data-driven option panels: describe a panel as [`Row`]s over a state resource and let
//! [`show_rows`] build the widgets, [`PanelPlugin`] keep them in step with the state and write
//! their edits back.
//!
//! Widgets carry [`Bound`] (which row and item they edit) plus a marker; plain-fn observers look the
//! row up in the [`Panel`] resource and call its setter, so no closure captures widget state.
//! Rows only rebuild their widgets when their [`Row::key`] changes, and never while a slider is
//! being dragged (a rebuild would end the drag).

mod model;

use std::marker::PhantomData;

use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy::ui::{Pressed, Selected};
use bevy::ui_widgets::{Activate, SliderValue, ValueChange};

pub use self::model::{Action, Get, Panel, Row, Run, Set, Toggle, pretty};
use crate::MugonUiSystems;
use crate::theme::GAP_SMALL;
use crate::widgets::{self as ui, TileSize, channel_slider, column, heading, hint, spawn_in, text_small, wrap};

/// Which row of the [`Panel`] a widget edits, and which of its items (a tile, a swatch, a colour channel).
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Bound {
    pub row: usize,
    pub item: usize,
}

#[derive(Component, Default, Clone)]
pub struct SliderBind;
#[derive(Component, Default, Clone)]
pub struct ChannelBind;
#[derive(Component, Default, Clone)]
pub struct TileBind;
#[derive(Component, Default, Clone)]
pub struct ToggleBind;
#[derive(Component, Default, Clone)]
pub struct SwatchBind;
#[derive(Component, Default, Clone)]
pub struct CustomBind;
#[derive(Component, Default, Clone)]
pub struct ActionBind;

/// A resource a panel can edit.
pub trait PanelState: Resource<Mutability = Mutable> {}

impl<T: Resource<Mutability = Mutable>> PanelState for T {}

/// Edits the state `S` through panel rows: observers and the value sync.
pub struct PanelPlugin<S: PanelState>(PhantomData<fn() -> S>);

impl<S: PanelState> Default for PanelPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S: PanelState> Plugin for PanelPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Panel<S>>().add_systems(Update, sync_values::<S>.in_set(MugonUiSystems::Sync));
    }
}

fn on_slider<S: PanelState>(change: On<ValueChange<f32>>, bound: Query<&Bound, With<SliderBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>) {
    let Ok(bound) = bound.get(change.source) else { return };
    if let Some(Row::Slider { set, .. }) = panel.rows.get(bound.row) {
        set(&mut state, change.value);
    }
}

fn on_channel<S: PanelState>(change: On<ValueChange<f32>>, bound: Query<&Bound, With<ChannelBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>) {
    let Ok(bound) = bound.get(change.source) else { return };
    if let Some(Row::Swatches { get, set, .. }) = panel.rows.get(bound.row) {
        let mut color = get(&state);
        color[bound.item] = change.value.round().clamp(0.0, 255.0) as u8;
        set(&mut state, color);
    }
}

fn on_tile<S: PanelState>(activate: On<Activate>, bound: Query<&Bound, With<TileBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>) {
    let Ok(bound) = bound.get(activate.entity) else { return };
    match panel.rows.get(bound.row) {
        Some(Row::Tiles { pick, .. }) | Some(Row::Pictures { pick, .. }) => pick(&mut state, bound.item),
        _ => {}
    }
}

fn on_toggle<S: PanelState>(activate: On<Activate>, bound: Query<&Bound, With<ToggleBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>) {
    let Ok(bound) = bound.get(activate.entity) else { return };
    if let Some(Row::Toggles(toggles)) = panel.rows.get(bound.row)
        && let Some(toggle) = toggles.get(bound.item)
    {
        let value = !(toggle.get)(&state);
        (toggle.set)(&mut state, value);
    }
}

fn on_swatch<S: PanelState>(activate: On<Activate>, bound: Query<&Bound, With<SwatchBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>) {
    let Ok(bound) = bound.get(activate.entity) else { return };
    if let Some(Row::Swatches { presets, set, .. }) = panel.rows.get(bound.row)
        && let Some(&color) = presets.get(bound.item)
    {
        set(&mut state, color);
    }
}

fn on_custom<S: PanelState>(activate: On<Activate>, bound: Query<&Bound, With<CustomBind>>, mut panel: ResMut<Panel<S>>) {
    let Ok(bound) = bound.get(activate.entity) else { return };
    panel.custom_open = if panel.custom_open == Some(bound.row) { None } else { Some(bound.row) };
    panel.built.clear();
}

fn on_action<S: PanelState>(activate: On<Activate>, bound: Query<&Bound, With<ActionBind>>, panel: Res<Panel<S>>, mut state: ResMut<S>, time: Res<Time>) {
    let Ok(bound) = bound.get(activate.entity) else { return };
    if let Some(Row::Actions(actions)) = panel.rows.get(bound.row)
        && let Some(action) = actions.get(bound.item)
        && action.enabled
    {
        (action.run)(&mut state, time.elapsed_secs_f64());
    }
}

fn bound_tile<S: PanelState>(label: String, bound: Bound, tile_size: TileSize) -> impl Scene {
    bsn! { @ui::tile(label, tile_size) bound TileBind on(on_tile::<S>) }
}

fn bound_toggle<S: PanelState>(label: &'static str, bound: Bound) -> impl Scene {
    bsn! { @ui::toggle_tile(label) bound ToggleBind on(on_toggle::<S>) }
}

fn bound_picture<S: PanelState>(image: Handle<Image>, cell: Rect, bound: Bound) -> impl Scene {
    bsn! { @ui::picture_tile(image, cell) bound TileBind on(on_tile::<S>) }
}

fn bound_swatch<S: PanelState>(color: [u8; 3], bound: Bound) -> impl Scene {
    bsn! { @ui::swatch(color) bound SwatchBind on(on_swatch::<S>) }
}

fn bound_custom<S: PanelState>(bound: Bound) -> impl Scene {
    bsn! { @ui::chip("Custom") bound CustomBind on(on_custom::<S>) }
}

fn bound_action<S: PanelState>(label: String, enabled: bool, bound: Bound) -> impl Scene {
    bsn! { @ui::button(label, enabled) bound ActionBind on(on_action::<S>) }
}

fn bound_slider<S: PanelState>(label: &'static str, ends: Option<(&'static str, &'static str)>, bound: Bound) -> impl Scene {
    ui::slider(label, ends, bsn! { bound SliderBind on(on_slider::<S>) })
}

fn bound_channel<S: PanelState>(label: &'static str, bound: Bound) -> impl Scene {
    channel_slider(label, bsn! { bound ChannelBind on(on_channel::<S>) })
}

/// Rebuilds the widgets under `content` (a container entity and its children) when the shape of
/// `rows` changed, unless a slider is being dragged; otherwise only stores the new rows.
/// `pictures` is the strip texture for [`Row::Pictures`].
pub fn show_rows<S: PanelState>(
    commands: &mut Commands,
    panel: &mut Panel<S>,
    rows: Vec<Row<S>>,
    content: Option<(Entity, Option<&Children>)>,
    pictures: Option<&Handle<Image>>,
    dragging: bool,
) {
    let keys: Vec<String> = rows.iter().map(Row::key).collect();
    let rebuild = keys != panel.built && !dragging;
    panel.rows = rows;
    if !rebuild {
        return;
    }
    let Some((container, children)) = content else {
        panel.built.clear();
        return;
    };
    for child in children.into_iter().flatten() {
        commands.entity(*child).despawn();
    }
    panel.built = keys;
    let custom_open = panel.custom_open;
    for (index, row) in panel.rows.iter().enumerate() {
        spawn_row::<S>(commands, container, index, row, custom_open == Some(index), pictures);
    }
}

fn spawn_row<S: PanelState>(commands: &mut Commands, parent: Entity, index: usize, row: &Row<S>, custom_open: bool, pictures: Option<&Handle<Image>>) {
    let bound = |item: usize| Bound { row: index, item };
    match row {
        Row::Heading(text) => {
            spawn_in(commands, parent, heading(text.clone()));
        }
        Row::Hint(text) => {
            spawn_in(commands, parent, hint(text.clone()));
        }
        Row::Slider { label, ends, .. } => {
            spawn_in(commands, parent, bound_slider::<S>(label, *ends, bound(0)));
        }
        Row::Tiles { labels, size, .. } => {
            let tiles = spawn_in(commands, parent, wrap(6.0));
            for (item, label) in labels.iter().enumerate() {
                spawn_in(commands, tiles, bound_tile::<S>(label.clone(), bound(item), *size));
            }
        }
        Row::Pictures { count, cell, .. } => {
            let tiles = spawn_in(commands, parent, wrap(4.0));
            let Some(image) = pictures else { return };
            for item in 0..*count {
                let rect = Rect::new((item * cell) as f32, 0.0, ((item + 1) * cell) as f32, *cell as f32);
                spawn_in(commands, tiles, bound_picture::<S>(image.clone(), rect, bound(item)));
            }
        }
        Row::Toggles(toggles) => {
            let tiles = spawn_in(commands, parent, wrap(6.0));
            for (item, toggle) in toggles.iter().enumerate() {
                spawn_in(commands, tiles, bound_toggle::<S>(toggle.label, bound(item)));
            }
        }
        Row::Swatches { label, presets, .. } => {
            let block = spawn_in(commands, parent, column(GAP_SMALL));
            spawn_in(commands, block, text_small(label.clone()));
            let swatches = spawn_in(commands, block, wrap(GAP_SMALL));
            for (item, &color) in presets.iter().enumerate() {
                spawn_in(commands, swatches, bound_swatch::<S>(color, bound(item)));
            }
            spawn_in(commands, swatches, bound_custom::<S>(bound(0)));
            if custom_open {
                for (channel, name) in ["Red", "Green", "Blue"].into_iter().enumerate() {
                    spawn_in(commands, block, bound_channel::<S>(name, bound(channel)));
                }
            }
        }
        Row::Actions(actions) => {
            let buttons = spawn_in(commands, parent, wrap(6.0));
            for (item, action) in actions.iter().enumerate() {
                spawn_in(commands, buttons, bound_action::<S>(action.label.clone(), action.enabled, bound(item)));
            }
        }
    }
}

/// True while a slider of a panel is being dragged: pass it to [`show_rows`].
pub fn is_dragging(sliders: &Query<(), (With<SliderBind>, With<Pressed>)>) -> bool {
    !sliders.is_empty()
}

type SliderFilter = Or<(With<SliderBind>, With<ChannelBind>)>;
type SliderState<'a> = (Entity, &'a Bound, &'a SliderValue, Has<SliderBind>, Has<ChannelBind>);
type ChoiceState<'a> = (Entity, &'a Bound, Has<Selected>, Option<&'a TileBind>, Option<&'a ToggleBind>, Option<&'a SwatchBind>, Option<&'a CustomBind>);

/// Keeps every bound widget showing what the state holds.
fn sync_values<S: PanelState>(
    state: Res<S>,
    panel: Res<Panel<S>>,
    sliders: Query<SliderState, SliderFilter>,
    choices: Query<ChoiceState>,
    mut commands: Commands,
) {
    for (entity, bound, value, is_slider, is_channel) in &sliders {
        let wanted = match panel.rows.get(bound.row) {
            Some(Row::Slider { get, .. }) if is_slider => get(&state),
            Some(Row::Swatches { get, .. }) if is_channel => get(&state)[bound.item] as f32,
            _ => continue,
        };
        if value.0 != wanted {
            commands.entity(entity).insert(SliderValue(wanted));
        }
    }
    for (entity, bound, selected, tile, toggle, swatch, custom) in &choices {
        let wanted = match panel.rows.get(bound.row) {
            Some(Row::Tiles { selected, .. }) | Some(Row::Pictures { selected, .. }) if tile.is_some() => Some(selected(&state) == bound.item),
            Some(Row::Toggles(toggles)) if toggle.is_some() => toggles.get(bound.item).map(|toggle| (toggle.get)(&state)),
            Some(Row::Swatches { presets, get, .. }) if swatch.is_some() => presets.get(bound.item).map(|&color| color == get(&state)),
            Some(Row::Swatches { presets, get, .. }) if custom.is_some() => Some(!presets.contains(&get(&state))),
            _ => None,
        };
        match wanted {
            Some(true) if !selected => {
                commands.entity(entity).insert(Selected);
            }
            Some(false) if selected => {
                commands.entity(entity).remove::<Selected>();
            }
            _ => {}
        }
    }
}
