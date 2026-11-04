/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Theme controller.
 *
 * Keeps the web UI and the native window chrome in sync. The native part is
 * handled by the `set_window_theme` Tauri command (GTK on Linux/BSD, native
 * titlebar on Windows/macOS).
 */

import { setWindowTheme } from '../invoke';

export type ThemeName = 'light' | 'dark';

const STORAGE_KEY = 'babilo:theme';

let current: ThemeName | null = null;

export function normalizeTheme(value: unknown): ThemeName {
  return value === 'light' ? 'light' : 'dark';
}

export function getTheme(): ThemeName {
  return current ?? 'dark';
}

/**
 * Applies a theme to the DOM and asks the native backend to update the window
 * decorations. Safe to call repeatedly.
 */
export function applyTheme(value: unknown): ThemeName {
  const theme = normalizeTheme(value);
  const root = document.documentElement;

  if (current !== theme || root.dataset.theme !== theme) {
    current = theme;
    root.dataset.theme = theme;
    root.style.colorScheme = theme;

    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      /* localStorage can be unavailable in hardened webviews */
    }

    void setWindowTheme(theme).catch((err) => {
      console.warn('[Babilo Theme] Failed to sync native window theme:', err);
    });
  }

  return theme;
}

/**
 * Applies the last known theme before the backend settings load, avoiding a
 * flash of the wrong palette during startup.
 */
export function initTheme(): void {
  let saved: string | null = null;
  try {
    saved = localStorage.getItem(STORAGE_KEY);
  } catch {
    /* ignore */
  }

  applyTheme(saved ?? 'dark');
}
