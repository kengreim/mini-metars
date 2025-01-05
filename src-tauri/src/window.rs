#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::profiles::{default_scale, ProfileWindowState};
use crate::settings::Settings;
use crate::MAIN_WINDOW_LABEL;
use anyhow::anyhow;
use log::debug;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowState {
    Maximized,
    FullScreen,
    Normal,
}

pub fn get_window_state(app: &AppHandle) -> Option<ProfileWindowState> {
    let w = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .map(|w| ProfileWindowState {
            state: if w.is_maximized().unwrap_or_default() {
                WindowState::Maximized
            } else if w.is_fullscreen().unwrap_or_default() {
                WindowState::FullScreen
            } else {
                WindowState::Normal
            },
            position: w.outer_position().ok(),
            size: w.outer_size().ok(),
            scale_factor: w.scale_factor().unwrap_or_else(|_| default_scale()),
        });
    debug!("Captured window state: {w:?}");

    w
}

pub fn apply_window_state(
    app: &AppHandle,
    window_state: &ProfileWindowState,
) -> Result<(), anyhow::Error> {
    app.get_webview_window(MAIN_WINDOW_LABEL).map_or_else(
        || Err(anyhow!("Could not find main window")),
        |w| {
            debug!("Applying window state: {window_state:?}");
            match window_state.state {
                WindowState::FullScreen => w.set_fullscreen(true)?,
                WindowState::Maximized => w.maximize()?,
                WindowState::Normal => {
                    if let Some(size) = window_state.size {
                        w.set_size(size)?;
                    }
                    if let Some(position) = window_state.position {
                        w.set_position(position)?;
                    }
                }
            }
            Ok(())
        },
    )
}

pub fn set_always_on_top_settings_checked(
    window: Option<&WebviewWindow>,
    settings: &Settings,
    always_on_top: bool,
) -> Result<(), String> {
    if settings.always_on_top() {
        debug!("Trying to set always on top to {always_on_top}");
        set_always_on_top(window, always_on_top)
    } else {
        debug!("Always on top not applied due to settings override");
        Ok(())
    }
}

fn set_always_on_top(window: Option<&WebviewWindow>, always_on_top: bool) -> Result<(), String> {
    window.map_or_else(
        || Err("Could not find window".to_string()),
        |w| {
            w.set_always_on_top(always_on_top)
                .map_err(|e| e.to_string())
        },
    )
}
