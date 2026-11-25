//! The Mugon platform's look and UI components for Bevy.
//!
//! Add [`MugonUiPlugin`] to an app to get the feathers widget set with the platform's theme
//! ([`theme`]). Component groups are opt-in Cargo features (`overlays`, `inputs`, `game-ui`,
//! `panel-model`, `test-harness`); see `examples/gallery.rs` for everything this crate offers.

pub mod device;
#[cfg(feature = "game-ui")]
pub mod game_ui;
pub mod icons;
#[cfg(feature = "tool-ui")]
pub mod tool_ui;
#[cfg(feature = "overlays")]
pub mod overlays;
#[cfg(feature = "inputs")]
pub mod inputs;
#[cfg(feature = "panel-model")]
pub mod panel_model;
#[cfg(all(feature = "test-harness", not(target_arch = "wasm32")))]
pub mod test_harness;
pub mod theme;
pub mod widgets;

use bevy::feathers::FeathersPlugins;
use bevy::feathers::theme::UiTheme;
use bevy::prelude::*;

/// Ordering of the systems that keep widgets in step with the state they show.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MugonUiSystems {
    /// Spawn and rebuild widgets here (or before this set).
    Build,
    /// Put systems that set `Selected`, `DisabledFlag` or slider values here (or before this set).
    Sync,
    /// Colours widgets from their state.
    Style,
}

/// Installs feathers and the Mugon theme.
pub struct MugonUiPlugin;

impl Plugin for MugonUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FeathersPlugins, device::DevicePlugin))
            .insert_resource(UiTheme(theme::create_theme()))
            .add_systems(PreStartup, icons::make_icons)
            .configure_sets(Update, (MugonUiSystems::Build, MugonUiSystems::Sync, MugonUiSystems::Style).chain())
            .add_systems(Update, (widgets::apply_disabled, widgets::style_choices).chain().in_set(MugonUiSystems::Style))
            .add_systems(Update, widgets::touch_scroll);
        #[cfg(feature = "inputs")]
        app.add_plugins(inputs::InputsPlugin);
        #[cfg(feature = "overlays")]
        app.add_plugins(overlays::OverlaysPlugin);
        #[cfg(feature = "game-ui")]
        app.add_plugins(game_ui::GameUiPlugin);
        #[cfg(feature = "tool-ui")]
        app.add_plugins(tool_ui::ToolUiPlugin);
    }
}

pub mod prelude {
    pub use crate::MugonUiPlugin;
    pub use crate::MugonUiSystems;
    pub use crate::widgets::*;
    #[cfg(feature = "inputs")]
    pub use crate::inputs::prelude::*;
    #[cfg(feature = "overlays")]
    pub use crate::overlays::prelude::*;
    #[cfg(feature = "game-ui")]
    pub use crate::game_ui::prelude::*;
    #[cfg(feature = "tool-ui")]
    pub use crate::tool_ui::prelude::*;
    pub use crate::device::Device;
    pub use crate::icons::Icons;
    pub use crate::theme::{GAP, GAP_SMALL, RADIUS, palette, token};
}
