//! Every component of `mugon-sdk-ui-bevy` on one screen: the static widgets in their states on the left,
//! and a live panel built from rows over a state resource on the right.
//!
//! `cargo run --features test-harness,panel-model,inputs,overlays,game-ui,tool-ui --example gallery`; with `MUGON_UI_SHOT=out.png` it saves a screenshot and exits.

use bevy::feathers::constants::{fonts, size};
use bevy::feathers::font_styles::InheritableFont;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemeBackgroundColor};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::text::FontWeight;
use bevy::ui::{ScrollPosition, Selected};
use bevy::ui_widgets::popover::PopoverSide;
use bevy::ui_widgets::{Activate, ScrollArea, ValueChange};
use mugon_sdk_ui_bevy::panel_model::{Action, Panel, PanelPlugin, Row, Toggle, is_dragging, show_rows};
use mugon_sdk_ui_bevy::prelude::*;
use mugon_sdk_ui_bevy::test_harness::{ScriptLog, TestHarnessPlugin};
use std::sync::Arc;

const GRID_ITEMS: usize = 9000;

const STYLES: [&str; 6] = ["Short", "Swept", "Spiky", "Bob", "Long", "Bun"];
const COLORS: [[u8; 3]; 6] = [[30, 24, 22], [70, 46, 32], [120, 78, 48], [176, 138, 84], [214, 184, 120], [230, 224, 214]];

#[derive(Resource, Default)]
struct Demo {
    style: usize,
    length: f32,
    shiny: bool,
    color: [u8; 3],
    shuffles: u32,
}

#[derive(Component, Default, Clone)]
struct DemoContent;
#[derive(Component, Default, Clone)]
struct OpenDialog;
#[derive(Component, Default, Clone)]
struct ShowToast;
#[derive(Component, Default, Clone)]
struct OpenPopover;
#[derive(Component, Default, Clone)]
struct AddLog;
#[derive(Component, Default, Clone)]
struct Animated;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MugonUiPlugin, PanelPlugin::<Demo>::default(), TestHarnessPlugin))
        .insert_resource(Demo { style: 1, length: 0.4, shiny: false, color: COLORS[2], shuffles: 0 })
        .add_observer(log_demo)
        .add_observer(on_activate)
        .add_observer(|change: On<ValueChange<bool>>| info!("INPUT checkbox {}", change.value))
        .add_observer(|change: On<ValueChange<Option<Entity>>>| info!("INPUT tab {}", change.value.is_some()))
        .add_systems(Startup, setup)
        .add_systems(Update, (show_demo.in_set(MugonUiSystems::Build), animate_progress))
        .add_observer(log_grid)
        .run();
}

/// What a script's `log` step prints, to check the panel's edits reached the state.
fn on_activate(
    activate: On<Activate>,
    dialogs: Query<(), With<OpenDialog>>,
    toasts: Query<(), With<ShowToast>>,
    popovers: Query<(), With<OpenPopover>>,
    add_logs: Query<(), With<AddLog>>,
    logs: Query<Entity, With<MessageLog>>,
    mut commands: Commands,
) {
    if dialogs.contains(activate.entity) {
        let body: Box<dyn SceneList> = Box::new(bsn_list! { @text_small("Delete the scene? This cannot be undone.".to_string()) });
        let actions: Box<dyn SceneList> = Box::new(bsn_list! { @button("Cancel".to_string(), true) -- @button("Delete".to_string(), true) });
        commands.queue_spawn_scene(dialog("Confirm".to_string(), 320.0, body, actions));
    }
    if toasts.contains(activate.entity) {
        commands.trigger(Toast::new("Saved"));
    }
    if add_logs.contains(activate.entity)
        && let Ok(log) = logs.single()
    {
        commands.trigger(LogMessage { log, text: "Player joined the lobby".to_string() });
    }
    if popovers.contains(activate.entity) {
        let content: Box<dyn SceneList> = Box::new(bsn_list! { @text_small("Anchored to its button".to_string()) });
        commands.queue_spawn_scene(popover(PopoverSide::Bottom, content)).insert(ChildOf(activate.entity));
    }
}

fn log_grid(_log: On<ScriptLog>, items: Query<&VirtualItem>) {
    info!("GRID items={} of {GRID_ITEMS}", items.iter().count());
}

fn tree_scene() -> Box<dyn SceneList> {
    let inner: Box<dyn SceneList> = Box::new(bsn_list! { @tree_leaf("Oak".to_string()) -- @tree_leaf("Pine".to_string()) });
    Box::new(bsn_list! {
        @tree_node("World".to_string(), true, inner)
        --
        @tree_node("Empty".to_string(), false, Box::new(bsn_list! { @tree_leaf("Hidden".to_string()) }))
        --
        @tree_leaf("Camera".to_string())
    })
}

fn grid_item(index: usize) -> Box<dyn SceneList> {
    Box::new(bsn_list! { @tile(index.to_string(), TileSize::Normal) Node { width: percent(100), height: percent(100) } })
}

fn menu_scene() -> impl Scene {
    let file: Box<dyn SceneList> = Box::new(bsn_list! {
        @menu_item("Open".to_string(), Some("Ctrl+O"))
        --
        @menu_divider()
        --
        @menu_item("Quit".to_string(), None)
    });
    bsn! { @menu_bar() Children [ @menu("File", file) ] }
}

fn log_demo(_log: On<ScriptLog>, demo: Res<Demo>, logs: Query<&Children, With<MessageLog>>) {
    info!("LOG lines={}", logs.iter().map(|children| children.len()).sum::<usize>());
    info!("DEMO style={} length={:.2} shiny={} color={:?} shuffles={}", demo.style, demo.length, demo.shiny, demo.color, demo.shuffles);
}

fn demo_rows() -> Vec<Row<Demo>> {
    vec![
        Row::Heading("Hairstyle".into()),
        Row::Tiles {
            labels: STYLES.iter().map(|style| style.to_string()).collect(),
            size: TileSize::Normal,
            selected: Arc::new(|demo| demo.style),
            pick: Arc::new(|demo, index| demo.style = index),
        },
        Row::Slider { label: "Length", ends: Some(("short", "long")), get: Arc::new(|demo| demo.length), set: Arc::new(|demo, value| demo.length = value) },
        Row::Toggles(vec![Toggle { label: "Shiny", get: Arc::new(|demo| demo.shiny), set: Arc::new(|demo, value| demo.shiny = value) }]),
        Row::Swatches { label: "Colour".into(), presets: COLORS.to_vec(), get: Arc::new(|demo| demo.color), set: Arc::new(|demo, value| demo.color = value) },
        Row::Hint("Edits go straight to the Demo resource.".into()),
        Row::Actions(vec![Action { label: "Shuffle".into(), enabled: true, run: Arc::new(|demo, _| demo.shuffles += 1) }, Action { label: "Disabled".into(), enabled: false, run: Arc::new(|_, _| {}) }]),
    ]
}

fn animate_progress(time: Res<Time>, mut bars: Query<&mut ProgressBar, With<Animated>>) {
    for mut bar in &mut bars {
        bar.0 = (time.elapsed_secs() * 0.25).fract();
    }
}

fn show_demo(
    mut commands: Commands,
    mut panel: ResMut<Panel<Demo>>,
    content: Query<(Entity, Option<&Children>), With<DemoContent>>,
    dragging: Query<(), (With<mugon_sdk_ui_bevy::panel_model::SliderBind>, With<bevy::ui::Pressed>)>,
) {
    show_rows(&mut commands, &mut panel, demo_rows(), content.single().ok(), None, is_dragging(&dragging));
}

fn setup(mut commands: Commands, icons: Res<Icons>) {
    commands.spawn(Camera2d);
    commands.queue_spawn_scene(bsn! {
        @hud_root()
        Children [
            @hud_anchor(HudAnchor::TopRight)
            Children [ @player_badge("HUD".to_string(), None) ]
        ]
    });
    let (undo, dice) = (icons.undo.clone(), icons.dice.clone());
    let make: MakeItem = Arc::new(grid_item);
    let grid = VirtualGrid { count: GRID_ITEMS, cell: Vec2::new(56.0, 28.0), gap: 4.0, make };
    let tooltip = Tooltip("A tooltip".to_string());
    commands.queue_spawn_scene(bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
            column_gap: px(GAP * 2.0),
            padding: px(GAP * 2.0),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        InheritableFont { font: fonts::REGULAR, font_size: size::MEDIUM_FONT, weight: FontWeight::NORMAL }
        Children [
            @panel()
            Node { width: px(420), row_gap: px(GAP), overflow: Overflow::scroll_y() }
            ScrollArea
            TouchScroll
            ScrollPosition
            Children [
                @title("mugon-sdk-ui-bevy".to_string())
                --
                @heading("Text".to_string())
                --
                @text_small("Small text in the main colour".to_string())
                --
                @hint("A hint in the dim colour".to_string())
                --
                @heading("Tiles: normal, selected, disabled".to_string())
                --
                @wrap(6.0)
                Children [
                    @tile("Normal".to_string(), TileSize::Normal)
                    --
                    @tile("Selected".to_string(), TileSize::Normal)
                    Selected
                    --
                    @button("Disabled".to_string(), false)
                ]
                --
                @heading("Toggle, chip, buttons".to_string())
                --
                @wrap(6.0)
                Children [
                    @toggle_tile("Toggle")
                    --
                    @toggle_tile("On")
                    Selected
                    --
                    @chip("Chip")
                    --
                    @button("Button".to_string(), true)
                    --
                    @icon_button(undo)
                    --
                    @icon_button(dice)
                ]
                --
                @heading("Swatches".to_string())
                --
                @wrap(4.0)
                Children [
                    @swatch([30, 24, 22])
                    --
                    @swatch([120, 78, 48])
                    Selected
                    --
                    @swatch([214, 184, 120])
                ]
                --
                @heading("Navigation".to_string())
                --
                Node { column_gap: px(2) }
                Children [
                    @nav_button("Body")
                    --
                    @nav_button("Face")
                    Selected
                    --
                    @nav_button("Hair")
                ]
                --
                @heading("Sliders".to_string())
                --
                @slider("Length", Some(("short", "long")), bsn! {})
                --
                @channel_slider("Red", bsn! {})
                --
                @divider()
                --
                @heading("Game UI".to_string())
                --
                @progress_bar(0.35)
                --
                @progress_bar(0.0)
                Animated
                --
                Node { column_gap: px(GAP), align_items: AlignItems::Center }
                Children [
                    @spinner(28.0)
                    --
                    @player_badge("Ben".to_string(), None)
                ]
                --
                @scoreboard_row(1, "Ben".to_string(), "1200".to_string(), true)
                --
                @scoreboard_row(2, "Sam".to_string(), "950".to_string(), false)
                --
                @message_log(64.0, 50)
            ]
            --
            @panel()
            Node { width: px(300), row_gap: px(GAP) }
            Children [
                @title("Inputs".to_string())
                --
                @divider()
                --
                @checkbox("Checkbox")
                --
                @select(&["Short", "Swept", "Spiky"], Some(1))
                --
                @text_field(None, Some("Search…"))
                --
                @number_input(2.5)
                --
                @tabs(&["Assets", "Scenes"], false)
                --
                @tabs(&["Day", "Night", "Dusk"], true)
                --
                @wrap(6.0)
                Children [
                    @button("Dialog".to_string(), true)
                    OpenDialog
                    --
                    @button("Toast".to_string(), true)
                    ShowToast
                    --
                    @button("Popover".to_string(), true)
                    OpenPopover
                    --
                    @button("Tooltip".to_string(), true)
                    tooltip
                    --
                    @button("Log".to_string(), true)
                    AddLog
                ]
            ]
            --
            @panel()
            Node { width: px(260), row_gap: px(GAP) }
            Children [
                @title("Tool UI".to_string())
                --
                @menu_scene()
                --
                @column(2.0)
                Children [ {tree_scene()} ]
                --
                @virtual_grid(200.0)
                grid
                --
                Node { height: px(36), flex_shrink: 0.0 }
                Children [
                    @tile("Resizable".to_string(), TileSize::Normal)
                    Node { width: px(120), flex_grow: 0.0 }
                    --
                    @splitter(SplitAxis::Horizontal, Resizes::Before, 60.0, 200.0)
                ]
            ]
            --
            @panel()
            Node { width: px(320), flex_grow: 0.0 }
            Children [
                @title("Panel model".to_string())
                --
                @divider()
                --
                @column(GAP)
                DemoContent
            ]
        ]
    });
}
