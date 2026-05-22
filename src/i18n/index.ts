/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import type { Locale, TranslationKey } from './strings';
import { translations } from './strings';
import type { LitElement } from 'lit';

// ── Reactive locale state ──
let currentLocale: Locale = 'en';
const listeners = new Set<(locale: Locale) => void>();

export function getLocale(): Locale { return currentLocale; }

export function setLocale(locale: Locale): void {
  if (currentLocale === locale) return;
  currentLocale = locale;
  listeners.forEach(cb => cb(locale));
  try { localStorage.setItem('babilo:locale', locale); } catch {}
}

export function subscribeLocale(callback: (locale: Locale) => void): () => void {
  listeners.add(callback);
  callback(currentLocale);
  return () => listeners.delete(callback);
}

export function t(key: TranslationKey, locale?: Locale): string {
  const loc = locale ?? currentLocale;
  const value = translations[loc]?.[key] ?? translations.en[key];
  if (value === undefined) {
    console.warn(`[i18n] Missing translation for key: "${key}"`);
    return key;
  }
  return value;
}

export function initI18n(): void {
  try {
    const saved = localStorage.getItem('babilo:locale') as Locale | null;
    if (saved && (saved === 'en' || saved === 'es')) {
      currentLocale = saved;
      return;
    }
    if (navigator.language.toLowerCase().startsWith('es')) {
      currentLocale = 'es';
    }
  } catch {}
}

// ── INTERFAZ para que TypeScript sepa que existe _t ──
export interface I18nMethods {
  _t(key: TranslationKey): string;
}

// ── Mixin simplificado (sin tipado genérico complejo) ──
export function withI18n<T extends new (...args: any[]) => LitElement>(Base: T) {
  return class extends Base implements I18nMethods {
    _i18nUnsubscribe?: () => void;

    connectedCallback() {
      super.connectedCallback?.();
      this._i18nUnsubscribe = subscribeLocale(() => {
        this.requestUpdate();
      });
    }

    disconnectedCallback() {
      this._i18nUnsubscribe?.();
      super.disconnectedCallback?.();
    }

    _t(key: TranslationKey): string {
      return t(key);
    }
  };
}