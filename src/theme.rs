//! The platform's look (Mantine, dark scheme, gray primary) as a feathers theme.

use bevy::color::Alpha;
use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::{SemanticToken, ThemeProps, ThemeToken};
use bevy::feathers::tokens;
use bevy::prelude::*;

pub mod token {
    use super::ThemeToken;

    pub const PANEL_BG: ThemeToken = ThemeToken::new_static("mugon.panel.bg");
    pub const PANEL_BORDER: ThemeToken = ThemeToken::new_static("mugon.panel.border");
    pub const DIVIDER: ThemeToken = ThemeToken::new_static("mugon.divider");
    pub const CHOICE_BG: ThemeToken = ThemeToken::new_static("mugon.choice.bg");
    pub const CHOICE_BG_HOVER: ThemeToken = ThemeToken::new_static("mugon.choice.bg.hover");
    pub const CHOICE_BG_PRESSED: ThemeToken = ThemeToken::new_static("mugon.choice.bg.pressed");
    pub const CHOICE_BG_SELECTED: ThemeToken = ThemeToken::new_static("mugon.choice.bg.selected");
    pub const CHOICE_BG_DISABLED: ThemeToken = ThemeToken::new_static("mugon.choice.bg.disabled");
    pub const CHOICE_TEXT: ThemeToken = ThemeToken::new_static("mugon.choice.text");
    pub const CHOICE_TEXT_SELECTED: ThemeToken = ThemeToken::new_static("mugon.choice.text.selected");
    pub const CHOICE_TEXT_DISABLED: ThemeToken = ThemeToken::new_static("mugon.choice.text.disabled");
    pub const CHOICE_BORDER_SELECTED: ThemeToken = ThemeToken::new_static("mugon.choice.border.selected");
    pub const SWATCH_RING: ThemeToken = ThemeToken::new_static("mugon.swatch.ring");
    pub const NAV_BG_HOVER: ThemeToken = ThemeToken::new_static("mugon.nav.bg.hover");
    pub const NAV_BG_SELECTED: ThemeToken = ThemeToken::new_static("mugon.nav.bg.selected");
}

/// Mantine's dark palette, and its gray shades with the blue tint removed (checked against the web
/// theme by `scripts/check-theme-parity.mjs`).
pub mod palette {
    use bevy::prelude::Color;

    pub const fn hex(rgb: u32) -> Color {
        Color::srgb(((rgb >> 16) & 0xff) as f32 / 255.0, ((rgb >> 8) & 0xff) as f32 / 255.0, (rgb & 0xff) as f32 / 255.0)
    }

    pub const DARK_0: Color = hex(0xc9c9c9);
    pub const DARK_2: Color = hex(0x828282);
    pub const DARK_3: Color = hex(0x696969);
    pub const DARK_4: Color = hex(0x424242);
    pub const DARK_5: Color = hex(0x3b3b3b);
    pub const DARK_6: Color = hex(0x2e2e2e);
    pub const DARK_7: Color = hex(0x242424);
    pub const DARK_8: Color = hex(0x1f1f1f);
    pub const GRAY_6: Color = hex(0x8a8a8a);
    pub const GRAY_7: Color = hex(0x4f4f4f);
    pub const GRAY_8: Color = hex(0x383838);
    pub const WHITE: Color = hex(0xffffff);
}

/// Corner radius of Mantine's `md`.
pub const RADIUS: f32 = 8.0;
/// The layout gap (not one of Mantine's spacing steps: the dense editor look wants 8 and 4 points).
pub const GAP: f32 = 8.0;
/// The small layout gap.
pub const GAP_SMALL: f32 = 4.0;

pub fn create_theme() -> ThemeProps {
    use palette::*;
    let mut theme = create_dark_theme();
    // Every token gets a semantic token of its own, so the dark theme's shared semantics and
    // per-surface overrides no longer apply to it.
    let mut set = |token: ThemeToken, color: Color| {
        let semantic = SemanticToken::new(token.to_string().into());
        theme.token_assignments.insert(token, semantic.clone());
        theme.semantic_base.insert(semantic, color);
    };

    set(tokens::WINDOW_BG, DARK_8);
    set(tokens::FOCUS_RING, GRAY_6.with_alpha(0.6));
    set(tokens::TEXT_MAIN, DARK_0);
    set(tokens::TEXT_DIM, DARK_2);

    set(tokens::BUTTON_BG, DARK_6);
    set(tokens::BUTTON_BG_HOVER, DARK_5);
    set(tokens::BUTTON_BG_PRESSED, DARK_4);
    set(tokens::BUTTON_BG_DISABLED, DARK_7);
    set(tokens::BUTTON_TEXT, DARK_0);
    set(tokens::BUTTON_TEXT_DISABLED, DARK_3);
    set(tokens::BUTTON_PRIMARY_BG, GRAY_7);
    set(tokens::BUTTON_PRIMARY_BG_HOVER, hex(0x5a5a5a));
    set(tokens::BUTTON_PRIMARY_BG_PRESSED, hex(0x666666));
    set(tokens::BUTTON_PRIMARY_BG_DISABLED, DARK_7);
    set(tokens::BUTTON_PRIMARY_TEXT, WHITE);
    set(tokens::BUTTON_PRIMARY_TEXT_DISABLED, DARK_3);
    set(tokens::BUTTON_PLAIN_BG, Color::NONE);
    set(tokens::BUTTON_PLAIN_BG_HOVER, DARK_6);
    set(tokens::BUTTON_PLAIN_BG_PRESSED, DARK_5);
    set(tokens::BUTTON_PLAIN_BG_DISABLED, Color::NONE);

    // The value text on top of the bar is hidden: the bar itself is the value.
    set(tokens::SLIDER_BG, DARK_5);
    set(tokens::SLIDER_BG_HOVER, hex(0x434343));
    set(tokens::SLIDER_BG_PRESSED, hex(0x484848));
    set(tokens::SLIDER_BG_DISABLED, DARK_6);
    set(tokens::SLIDER_BAR, hex(0xd4d4d4));
    set(tokens::SLIDER_BAR_HOVER, hex(0xe4e4e4));
    set(tokens::SLIDER_BAR_PRESSED, WHITE);
    set(tokens::SLIDER_BAR_DISABLED, DARK_4);
    set(tokens::SLIDER_TEXT, Color::NONE);
    set(tokens::SLIDER_TEXT_DISABLED, Color::NONE);

    set(tokens::SCROLLBAR_BG, Color::NONE);
    set(tokens::SCROLLBAR_THUMB, DARK_4);
    set(tokens::SCROLLBAR_THUMB_HOVER, DARK_3);


    // Checkbox: a dark box that fills grey when checked, white mark.
    for (token, color) in [
        (tokens::CHECKBOX_BG, DARK_6),
        (tokens::CHECKBOX_BG_HOVER, DARK_5),
        (tokens::CHECKBOX_BG_PRESSED, DARK_4),
        (tokens::CHECKBOX_BG_DISABLED, DARK_7),
        (tokens::CHECKBOX_BG_CHECKED, GRAY_7),
        (tokens::CHECKBOX_BG_CHECKED_HOVER, hex(0x5a5a5a)),
        (tokens::CHECKBOX_BG_CHECKED_PRESSED, hex(0x666666)),
        (tokens::CHECKBOX_BG_CHECKED_DISABLED, DARK_7),
        (tokens::CHECKBOX_BORDER, DARK_4),
        (tokens::CHECKBOX_BORDER_HOVER, DARK_3),
        (tokens::CHECKBOX_BORDER_PRESSED, DARK_3),
        (tokens::CHECKBOX_BORDER_DISABLED, DARK_5),
        (tokens::CHECKBOX_BORDER_CHECKED, GRAY_6),
        (tokens::CHECKBOX_BORDER_CHECKED_HOVER, GRAY_6),
        (tokens::CHECKBOX_BORDER_CHECKED_PRESSED, GRAY_6),
        (tokens::CHECKBOX_BORDER_CHECKED_DISABLED, DARK_5),
        (tokens::CHECKBOX_MARK, WHITE),
        (tokens::CHECKBOX_MARK_DISABLED, DARK_3),
        (tokens::CHECKBOX_TEXT, DARK_0),
        (tokens::CHECKBOX_TEXT_DISABLED, DARK_3),
    ] {
        set(token, color);
    }

    // Text fields and number inputs.
    set(tokens::TEXT_INPUT_BG, DARK_6);
    set(tokens::TEXT_INPUT_LABEL_BG, DARK_4);
    set(tokens::TEXT_INPUT_TEXT, DARK_0);
    set(tokens::TEXT_INPUT_TEXT_DISABLED, DARK_3);
    set(tokens::TEXT_INPUT_CURSOR, WHITE);
    set(tokens::TEXT_INPUT_SELECTION, GRAY_7);
    set(tokens::TEXT_INPUT_SELECTION_UNFOCUSED, DARK_5);

    // Menus, dropdowns and lists.
    set(tokens::MENU_BG, DARK_7);
    set(tokens::MENU_BORDER, DARK_4);
    set(tokens::MENUITEM_BG, Color::NONE);
    set(tokens::MENUITEM_BG_HOVER, DARK_5);
    set(tokens::MENUITEM_BG_PRESSED, DARK_4);
    set(tokens::MENUITEM_BG_FOCUSED, DARK_5);
    set(tokens::MENUITEM_TEXT, DARK_0);
    set(tokens::MENUITEM_TEXT_DISABLED, DARK_3);
    set(tokens::LISTROW_BG, Color::NONE);
    set(tokens::LISTROW_BG_HOVER, DARK_5);
    set(tokens::LISTROW_BG_SELECTED, GRAY_7);
    set(tokens::LISTROW_TEXT, DARK_0);
    set(tokens::LISTROW_TEXT_DISABLED, DARK_3);

    // Dialogs.
    set(tokens::DIALOG_BG, DARK_7);
    set(tokens::DIALOG_BORDER, DARK_4);
    set(tokens::DIALOG_HEADER_BG, DARK_8);
    set(tokens::DIALOG_TEXT, DARK_0);
    set(tokens::DIALOG_HEADER_TEXT, WHITE);

    set(token::PANEL_BG, DARK_7.with_alpha(0.94));
    set(token::PANEL_BORDER, DARK_4);
    set(token::DIVIDER, DARK_5);
    set(token::CHOICE_BG, DARK_6);
    set(token::CHOICE_BG_HOVER, DARK_5);
    set(token::CHOICE_BG_PRESSED, DARK_4);
    set(token::CHOICE_BG_SELECTED, GRAY_7);
    set(token::CHOICE_BG_DISABLED, DARK_7);
    set(token::CHOICE_TEXT, DARK_0);
    set(token::CHOICE_TEXT_SELECTED, WHITE);
    set(token::CHOICE_TEXT_DISABLED, DARK_3);
    set(token::CHOICE_BORDER_SELECTED, GRAY_6);
    set(token::SWATCH_RING, WHITE);
    set(token::NAV_BG_HOVER, DARK_6);
    set(token::NAV_BG_SELECTED, GRAY_8);
    theme
}
