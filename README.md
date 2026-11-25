# mugon-sdk-ui-bevy

The Mugon platform's look and UI components for Bevy, built on `bevy_feathers` and BSN. A game or tool adds
`MugonUiPlugin` and gets the platform theme (Mantine dark, gray primary) and a set of themed widgets.

```rust
app.add_plugins((DefaultPlugins, mugon_sdk_ui_bevy::MugonUiPlugin));
```

Run the gallery of every component with `cargo run --example gallery`.

## Features

Nothing is enabled by default, to keep wasm builds small.

| Feature | Adds |
| --- | --- |
| `overlays` | `dialog` (modal), `Tooltip` component, `popover`, `Toast` event |
| `inputs` | `checkbox`, `select` (dropdown), `text_field` (with placeholder), `number_input`, `tabs` (tab strip or segmented control) |
| `game-ui` | `progress_bar`, `spinner`, `player_badge`, `scoreboard_row`, `message_log` (trigger `LogMessage`), `hud_root` + `hud_anchor`, and the `SafeArea` resource (set `MUGON_UI_SAFE_AREA=top,right,bottom,left` to try insets natively). Use `device::game_window()` as the primary window on the web: Bevy's default 1280x720 canvas is cropped on phones, hiding controls at the right or bottom edge. Pin controls with `hud_anchor`s, not pixel offsets |
| `tool-ui` | `menu_bar` + `menu` / `menu_item` / `menu_divider`, `splitter` (drag to resize a sibling), `tree_node` / `tree_leaf`, and `virtual_grid` (`VirtualGrid`: thousands of items, only the visible ones exist as entities) |
| `panel-model` | generic data-driven option panels (`Row<S>`) |
| `test-harness` | scripted pointer input and screenshots for UI tests |

## Compatibility

| `mugon-sdk-ui-bevy` | Bevy |
| --- | --- |
| 0.1 | 0.20 |

Feathers is experimental and changes between Bevy releases, so this crate is tied to one Bevy version at a time.

Design: [ADR 0011](../../docs/ADRs/0011-shared-bevy-ui-and-controls-crates.md).
