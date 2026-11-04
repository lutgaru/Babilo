/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Native desktop integration.
//!
//! The web UI always renders Babilo's palette, but the native window chrome is
//! drawn by the OS. This module keeps that chrome in sync with the app theme:
//! GTK client-side decorations on Linux/BSD and native titlebars elsewhere.

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub mod gtk_window;

/// Installs any platform-specific window chrome once, right after the window
/// is created.
pub fn setup_window(window: &tauri::WebviewWindow, theme: &str) -> tauri::Result<()> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    gtk_window::install_titlebar(window)?;

    set_window_theme(window, theme)
}

/// Applies a theme to the native window decorations.
///
/// Tauri maps this to `gtk-application-prefer-dark-theme` on Linux/BSD and to
/// the native titlebar theme on Windows/macOS. On GTK desktops we additionally
/// repaint and compact the headerbar so it matches the web UI instead of the
/// system GTK theme.
pub fn set_window_theme(window: &tauri::WebviewWindow, theme: &str) -> tauri::Result<()> {
    let dark = matches!(theme, "dark");
    let native_theme = if dark {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    };

    window.set_theme(Some(native_theme))?;

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    gtk_window::apply_titlebar_theme(dark)?;

    Ok(())
}
