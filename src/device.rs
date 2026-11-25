//! What the app is running on: phone or not, and how zoomed out the page around it is.

use bevy::prelude::*;
use bevy::ui::UiScale;

/// What the game is running on, judged from the device rather than from the size of the game's
/// frame: a host page without a mobile viewport meta tag lays out 980 CSS px even on a 390 px
/// wide phone, so the frame alone looks like a desktop window.
#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub struct Device {
    /// A phone (touch input, small screen): gets the touch layouts whatever the frame size.
    pub phone: bool,
    /// How much the page is zoomed out on a phone, e.g. 2.5 for 980 CSS px on a 390 px phone. The UI
    /// is scaled up by this so controls stay finger-sized. 1 on desktops and on properly configured
    /// pages.
    pub zoom: f32,
    /// What the browser reported, logged to see why a device counts as a phone or not.
    pub signals: Option<Signals>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Signals {
    pub coarse_pointer: bool,
    pub any_coarse_pointer: bool,
    pub max_touch_points: i32,
    pub mobile_user_agent: bool,
    pub screen: (f32, f32),
    pub inner: (f32, f32),
}

#[cfg(target_arch = "wasm32")]
pub fn device() -> Device {
    // Phones have a short screen side below this many CSS px; tablets are at 600 and up.
    const PHONE_SCREEN: f32 = 600.0;
    let Some(w) = web_sys::window() else { return Device { phone: false, zoom: 1.0, signals: None } };
    let media = |q: &str| w.match_media(q).ok().flatten().is_some_and(|m| m.matches());
    let nav = w.navigator();
    let ua = nav.user_agent().unwrap_or_default();
    let screen = w.screen().ok().map(|s| (s.width().unwrap_or(0) as f32, s.height().unwrap_or(0) as f32)).unwrap_or_default();
    let px = |v: Result<wasm_bindgen::JsValue, _>| v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let s = Signals {
        coarse_pointer: media("(pointer: coarse)"),
        any_coarse_pointer: media("(any-pointer: coarse)"),
        max_touch_points: nav.max_touch_points(),
        // Phones only: Android tablets leave out "Mobile", iPads in mobile mode say "iPad".
        mobile_user_agent: (ua.contains("iPhone") || ua.contains("iPod") || ua.contains("Mobile")) && !ua.contains("iPad"),
        screen,
        inner: (px(w.inner_width()), px(w.inner_height())),
    };
    let touch = s.coarse_pointer || s.any_coarse_pointer || s.max_touch_points > 0 || s.mobile_user_agent;
    let ((sw, sh), (iw, ih)) = (s.screen, s.inner);
    // The screen size is the real one on phones, but Chrome's device emulation leaves it at the
    // monitor size inside a cross-origin iframe (the game's frame on Mugon); a phone user agent
    // stands in for it there.
    let small_screen = sw > 0.0 && sh > 0.0 && sw.min(sh) < PHONE_SCREEN;
    if !touch || !(small_screen || s.mobile_user_agent) {
        return Device { phone: false, zoom: 1.0, signals: Some(s) };
    }
    // Short side of the device: the screen's if it is believable, else a typical phone's.
    let device_short = if small_screen { sw.min(sh) } else { 400.0 };
    let zoom = iw.min(ih) / device_short;
    Device { phone: true, zoom: if zoom > 1.15 { zoom.min(3.0) } else { 1.0 }, signals: Some(s) }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn device() -> Device {
    // Dev: MUGON_UI_PHONE=1 forces the phone layouts, MUGON_UI_ZOOM=2.5 simulates a zoomed-out phone page.
    Device {
        phone: std::env::var("MUGON_UI_PHONE").is_ok_and(|v| v == "1"),
        zoom: std::env::var("MUGON_UI_ZOOM").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0),
        signals: None,
    }
}

/// A window whose canvas fills the page on the web (Bevy's default is a fixed 1280x720 canvas that a phone
/// crops, hiding controls placed at the right or bottom edge): `WindowPlugin { primary_window: Some(game_window()), .. }`.
/// Lay controls out with `hud_anchor`s and percentages instead of pixel offsets from the top-left.
pub fn game_window() -> Window {
    Window { fit_canvas_to_parent: true, ..default() }
}

/// Keeps the [`Device`] resource current and scales the UI by its page zoom, so controls stay finger-sized.
pub(crate) struct DevicePlugin;

impl Plugin for DevicePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(device()).add_systems(PreUpdate, update_device);
    }
}

fn update_device(mut current: ResMut<Device>, mut ui_scale: ResMut<UiScale>) {
    let device = device();
    if *current != device {
        info!("ui device: {device:?}");
        *current = device;
    }
    if (ui_scale.0 - device.zoom).abs() > 0.01 {
        ui_scale.0 = device.zoom;
    }
}
