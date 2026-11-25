//! What a panel shows, as plain data: a list of rows with getters and setters on a state resource
//! `S`. A tool builds rows from its state, [`super::show_rows`] renders them, and nothing in here
//! knows about the UI toolkit.

use std::sync::Arc;

use bevy::prelude::*;

use crate::widgets::TileSize;

pub type Get<S, T> = Arc<dyn Fn(&S) -> T + Send + Sync>;
pub type Set<S, T> = Arc<dyn Fn(&mut S, T) + Send + Sync>;
/// Gets the elapsed time in seconds, for seeding randomness.
pub type Run<S> = Arc<dyn Fn(&mut S, f64) + Send + Sync>;

pub struct Toggle<S> {
    pub label: &'static str,
    pub get: Get<S, bool>,
    pub set: Set<S, bool>,
}

pub struct Action<S> {
    pub label: String,
    pub enabled: bool,
    pub run: Run<S>,
}

pub enum Row<S> {
    Heading(String),
    Hint(String),
    Slider { label: &'static str, ends: Option<(&'static str, &'static str)>, get: Get<S, f32>, set: Set<S, f32> },
    Tiles { labels: Vec<String>, size: TileSize, selected: Get<S, usize>, pick: Set<S, usize> },
    /// Tiles with a picture each, cut from a strip of `count` square cells of `cell` texels.
    Pictures { count: usize, cell: usize, selected: Get<S, usize>, pick: Set<S, usize> },
    Toggles(Vec<Toggle<S>>),
    Swatches { label: String, presets: Vec<[u8; 3]>, get: Get<S, [u8; 3]>, set: Set<S, [u8; 3]> },
    Actions(Vec<Action<S>>),
}

impl<S: 'static> Row<S> {
    /// Identifies the shape of the row: when no row's key changed, the widgets already match and
    /// only their values need refreshing.
    pub fn key(&self) -> String {
        match self {
            Row::Heading(text) => format!("h:{text}"),
            Row::Hint(text) => format!("i:{text}"),
            Row::Slider { label, ends, .. } => format!("s:{label}:{ends:?}"),
            Row::Tiles { labels, size, .. } => format!("t:{size:?}:{}", labels.join("|")),
            Row::Pictures { count, cell, .. } => format!("d:{count}:{cell}"),
            Row::Toggles(toggles) => format!("g:{}", toggles.iter().map(|toggle| toggle.label).collect::<Vec<_>>().join("|")),
            Row::Swatches { label, presets, .. } => format!("w:{label}:{}", presets.len()),
            Row::Actions(actions) => format!("a:{}", actions.iter().map(|action| format!("{}{}", action.label, action.enabled)).collect::<Vec<_>>().join("|")),
        }
    }

    pub fn tiles<T: Copy + PartialEq + Send + Sync + 'static>(all: &'static [T], name: fn(T) -> &'static str, get: fn(&S) -> T, set: fn(&mut S, T)) -> Row<S> {
        Row::Tiles {
            labels: all.iter().map(|&value| pretty(name(value))).collect(),
            size: TileSize::Normal,
            selected: Arc::new(move |state| all.iter().position(|&value| value == get(state)).unwrap_or(0)),
            pick: Arc::new(move |state, index| set(state, all[index])),
        }
    }
}

/// A panel's rows and what is currently built from them.
#[derive(Resource)]
pub struct Panel<S: Send + Sync + 'static> {
    pub rows: Vec<Row<S>>,
    /// Keys of the rows the widgets were built from.
    pub built: Vec<String>,
    /// Which swatch row has its custom colour sliders open.
    pub custom_open: Option<usize>,
}

impl<S: Send + Sync + 'static> Default for Panel<S> {
    fn default() -> Self {
        Self { rows: Vec::new(), built: Vec::new(), custom_open: None }
    }
}

/// "HighBoots" -> "High boots".
pub fn pretty(name: &str) -> String {
    let mut out = String::new();
    for (index, letter) in name.chars().enumerate() {
        if index > 0 && letter.is_uppercase() {
            out.push(' ');
            out.extend(letter.to_lowercase());
        } else {
            out.push(letter);
        }
    }
    out
}
