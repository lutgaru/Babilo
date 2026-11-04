/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! GTK titlebar integration for Linux and BSD desktops.
//!
//! Tao (Tauri's windowing layer) creates a `GtkHeaderBar` on Wayland, but it
//! inherits the system GTK theme verbatim: on GNOME that means a 47 px tall bar
//! whose colours can differ from Babilo's dark UI. On X11 the titlebar is drawn
//! by the window manager instead. This module normalises both cases by giving
//! every backend a compact, theme-aware client-side headerbar.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{gdk, CssProvider, HeaderBar, StyleContext};
use tauri::WebviewWindow;

thread_local! {
    /// A single provider per process. Updating its contents re-styles every
    /// window without stacking duplicate providers.
    static TITLEBAR_CSS: RefCell<Option<CssProvider>> = const { RefCell::new(None) };
}

const DARK_CSS: &str = r#"
headerbar {
  min-height: 30px;
  padding: 0 6px;
  background-color: #111118;
  background-image: none;
  border: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  box-shadow: none;
  color: #eeeef2;
}
headerbar:backdrop {
  background-color: #111118;
  background-image: none;
  color: #8080a0;
}
headerbar .title,
headerbar:backdrop .title {
  color: #eeeef2;
  font-size: 12px;
  font-weight: 600;
}
headerbar button {
  min-height: 24px;
  min-width: 24px;
  padding: 0;
  color: #eeeef2;
}
headerbar:backdrop button {
  color: #8080a0;
}
"#;

const LIGHT_CSS: &str = r#"
headerbar {
  min-height: 30px;
  padding: 0 6px;
  background-color: #f0f0f5;
  background-image: none;
  border: none;
  border-bottom: 1px solid rgba(0, 0, 0, 0.07);
  box-shadow: none;
  color: #111118;
}
headerbar:backdrop {
  background-color: #f0f0f5;
  background-image: none;
  color: #606080;
}
headerbar .title,
headerbar:backdrop .title {
  color: #111118;
  font-size: 12px;
  font-weight: 600;
}
headerbar button {
  min-height: 24px;
  min-width: 24px;
  padding: 0;
  color: #111118;
}
headerbar:backdrop button {
  color: #606080;
}
"#;

/// Installs our compact headerbar on X11 and adopts tao's headerbar on
/// Wayland. Also respects the desktop's configured window-button layout.
pub fn install_titlebar(window: &WebviewWindow) -> tauri::Result<()> {
    install_css_provider();

    let gtk_window = window.gtk_window()?;
    let decoration_layout = system_decoration_layout();

    if let Some(titlebar) = gtk_window.titlebar() {
        // Wayland: tao already installed an EventBox + HeaderBar. Keep that
        // structure untouched (it is what provides native drag/snap and the
        // window controls) and only restyle the headerbar.
        if let Some(header) = find_headerbar(&titlebar) {
            // GtkHeaderBar always reserves room for a subtitle in its sizing
            // label, which adds ~10px even when none is displayed.
            header.set_has_subtitle(false);
            if let Some(layout) = decoration_layout.as_deref() {
                header.set_decoration_layout(Some(layout));
            }
        }
        return Ok(());
    }

    // X11/other backends: the window manager owns the titlebar unless we opt
    // into client-side decorations. A HeaderBar keeps native controls on every
    // desktop while letting us match Babilo's palette.
    let title = window.title().unwrap_or_else(|_| "Babilo".into());
    let header = HeaderBar::builder()
        .show_close_button(true)
        .title(title)
        .build();
    header.set_has_subtitle(false);
    if let Some(layout) = decoration_layout.as_deref() {
        header.set_decoration_layout(Some(layout));
    }

    gtk_window.set_titlebar(Some(&header));
    // The window was already shown by the time Tauri's setup hook runs, so the
    // newly installed titlebar has to be made visible explicitly.
    header.show_all();
    Ok(())
}

/// Repaints the headerbar to match the active Babilo theme.
pub fn apply_titlebar_theme(dark: bool) -> tauri::Result<()> {
    let css = if dark { DARK_CSS } else { LIGHT_CSS };

    TITLEBAR_CSS.with(|slot| {
        if let Some(provider) = slot.borrow().as_ref() {
            if let Err(err) = provider.load_from_data(css.as_bytes()) {
                eprintln!("⚠️ Failed to apply GTK titlebar CSS: {err}");
            }
        }
    });

    Ok(())
}

fn install_css_provider() {
    TITLEBAR_CSS.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }

        let provider = CssProvider::new();
        if let Some(screen) = gdk::Screen::default() {
            StyleContext::add_provider_for_screen(
                &screen,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        *slot.borrow_mut() = Some(provider);
    });
}

/// Reads the desktop's preferred button order (e.g. close on the left for some
/// GNOME/Ubuntu configurations) instead of hardcoding GNOME's right-side order.
fn system_decoration_layout() -> Option<String> {
    gtk::Settings::default()
        .and_then(|settings| settings.gtk_decoration_layout())
        .map(|layout| layout.to_string())
        .filter(|layout| !layout.is_empty())
}

fn find_headerbar(widget: &gtk::Widget) -> Option<HeaderBar> {
    if let Some(header) = widget.downcast_ref::<HeaderBar>() {
        return Some(header.clone());
    }

    let container = widget.downcast_ref::<gtk::Container>()?;
    container
        .children()
        .into_iter()
        .find_map(|child| find_headerbar(&child))
}
