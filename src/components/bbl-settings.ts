/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html } from 'lit';
import { property, state, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { loadSettings, saveSettings } from '../invoke';
import type { AppSettings } from '../types/babilo';
import './settings/settings-audio';
import './settings/settings-model';
import './settings/settings-appearance';

type SettingsSection = 'audio' | 'model' | 'appearance';

@customElement('bbl-settings')
export class BblSettings extends withI18n(LitElement) {

    @property({ type: Boolean, reflect: true })
    open = false;

    @state()
    private _section: SettingsSection = 'audio';

    @state()
    private _settings: AppSettings | null = null;

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    updated(changed: Map<string, unknown>) {
        if (changed.has('open') && this.open) {
            this._initSettings();
        }
    }

    private async _initSettings() {
        try {
            this._settings = await loadSettings();
        } catch (e) {
            console.error('Failed to load settings:', e);
        }
    }

    private async _updateSettings(partial: Partial<AppSettings>) {
        if (!this._settings) return;
        const next: AppSettings = { ...this._settings, ...partial };
        this._settings = next;
        try {
            await saveSettings(next);
        } catch (e) {
            console.error('Failed to save settings:', e);
        }
    }

    private _updateSection<K extends keyof AppSettings>(section: K, partial: Partial<AppSettings[K]>) {
        if (!this._settings) return;
        const current = this._settings[section] ?? {} as AppSettings[K];
        this._updateSettings({ [section]: { ...current, ...partial } } as Partial<AppSettings>);
    }

    private _close() {
        this.dispatchEvent(new CustomEvent('settings-close', { bubbles: true, composed: true }));
    }

    private _navItem(id: SettingsSection, label: string, icon: unknown) {
        const active = this._section === id;
        return html`
      <button
        class="flex items-center gap-1.5 px-3 py-3 rounded-full text-xs font-medium
               transition-[background,color] duration-150
               ${active ? 'bg-bbl-accent2 text-white' : 'text-bbl-text-muted'}"
        @click=${() => { this._section = id; }}>
        ${icon}
        ${label}
      </button>
    `;
    }

    private _renderSection() {
        const s = this._settings;
        if (!s) return html``;

        switch (this._section) {
            case 'audio':
                return html`
              <bbl-settings-audio
                .data=${s.tts}
                .onChange=${(p: Partial<typeof s.tts>) => this._updateSection('tts', p)}>
              </bbl-settings-audio>`;
            case 'model':
                return html`
              <bbl-settings-model
                .data=${s as unknown as Record<string, unknown>}
                .onChange=${(section: string, p: Record<string, unknown>) => this._updateSection(section as keyof AppSettings, p)}>
              </bbl-settings-model>`;
            case 'appearance':
                return html`
              <bbl-settings-appearance
                .data=${s.gui}
                .onChange=${(p: Partial<typeof s.gui>) => this._updateSection('gui', p)}>
              </bbl-settings-appearance>`;
        }
    }

    render() {
        const micIcon = html`
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor"
           stroke-width="1.7" stroke-linecap="round">
        <path d="M12 2a4 4 0 0 0-4 4v6a4 4 0 0 0 8 0V6a4 4 0 0 0-4-4z"/>
        <path d="M6 10v2a6 6 0 0 0 12 0v-2"/>
      </svg>`;

        const modelIcon = html`
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor"
           stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 4h16v16H4z"/>
        <path d="M8 12h4l2-3 2 6 2-3"/>
      </svg>`;

        const appearanceIcon = html`
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor"
           stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="5"/>
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>`;

        return html`
      <div class="fixed inset-0 z-[100] flex flex-col bg-bbl-bg
                  transition-[transform] duration-[280ms] ease-[cubic-bezier(0.32,0.72,0,1)]
                  will-change-transform
                  ${this.open ? 'translate-y-0' : 'translate-y-full'}"
           role="dialog" aria-modal="true"
           aria-label="${this._t('settings.title')}">

        <header class="flex items-center justify-between
                       py-2 px-5 flex-shrink-0
                       border-b border-bbl-border">
          <span class="text-sm font-semibold text-bbl-text">
            ${this._t('settings.title')}
          </span>

          <nav class="flex items-center gap-1 overflow-x-auto max-w-[60vw]">
            ${this._navItem('audio', this._t('settings.nav.audio'), micIcon)}
            ${this._navItem('model', this._t('settings.nav.model'), modelIcon)}
            ${this._navItem('appearance', this._t('settings.nav.appearance'), appearanceIcon)}
          </nav>

          <button
            @click=${this._close}
            aria-label="${this._t('settings.close')}"
            class="w-8 h-8 rounded-full flex items-center justify-center
                   bg-bbl-btn-bg text-bbl-text-muted
                   hover:bg-bbl-btn-hover hover:text-bbl-text
                   transition-[background,color] duration-150">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
              <path d="M18 6 6 18M6 6l12 12"/>
            </svg>
          </button>
        </header>

        <div class="flex-1 overflow-y-auto pb-8">
          ${this._renderSection()}
        </div>

      </div>
    `;
    }
}
