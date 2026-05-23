/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html } from 'lit';
import { property, state, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

type SettingsSection = 'audio' | 'language' | 'appearance';

@customElement('bbl-settings')
export class BblSettings extends withI18n(LitElement) {

    @property({ type: Boolean, reflect: true })
    open = false;

    @state()
    private _section: SettingsSection = 'audio';

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
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

    private _row(label: string, sublabel: string, control: unknown) {
        return html`
      <div class="flex items-center justify-between
                  px-5 py-3.5 rounded-xl gap-4
                  transition-[background] duration-150 hover:bg-bbl-btn-hover">
        <div class="flex flex-col gap-0.5 min-w-0">
          <span class="text-sm text-bbl-text leading-snug">${label}</span>
          <span class="text-xs text-bbl-text-faint leading-snug truncate">${sublabel}</span>
        </div>
        <div class="flex-shrink-0">${control}</div>
      </div>
    `;
    }

    private _sectionHeader(title: string) {
        return html`
      <p class="px-5 pt-5 pb-1 text-[10px] font-semibold uppercase tracking-[0.1em]
                text-bbl-text-faint">
        ${title}
      </p>
    `;
    }

    private _audioSection() {
        const deviceSelect = (id: string) => html`
      <div class="relative">
        <select id=${id}
          class="appearance-none bg-bbl-surface border border-bbl-border text-bbl-text
                 rounded-bbl-sm px-2.5 py-1.5 text-xs cursor-pointer outline-none
                 focus:border-bbl-accent2 pr-7">
          <option>${this._t('settings.audio.default')}</option>
        </select>
        <svg class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2
                    w-3 h-3 text-bbl-text-muted"
             viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 9l6 6 6-6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    `;

        const slider = () => html`
      <div class="w-28">
        <input type="range" min="0" max="100" value="80"
          class="appearance-none w-full h-1 rounded bg-bbl-btn-bg border border-bbl-border
                 outline-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none
                 [&::-webkit-slider-thumb]:w-4
                 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full
                 [&::-webkit-slider-thumb]:bg-bbl-accent2
                 [&::-webkit-slider-thumb]:shadow-[0_1px_4px_rgba(0,0,0,0.25)]
                 [&::-webkit-slider-thumb]:transition-transform
                 [&::-webkit-slider-thumb]:duration-150
                 [&::-webkit-slider-thumb]:hover:scale-110">
      </div>
    `;

        return html`
      ${this._sectionHeader(this._t('settings.audio.input_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${this._row(
            this._t('settings.audio.microphone'),
            this._t('settings.audio.microphone_sub'),
            deviceSelect('mic-select')
        )}
      </div>

      ${this._sectionHeader(this._t('settings.audio.output_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${this._row(
            this._t('settings.audio.output_volume'),
            this._t('settings.audio.output_volume_sub'),
            slider()
        )}
      </div>
    `;
    }

    private _languageSection() {
        const langSelect = html`
      <div class="relative">
        <select
          class="appearance-none bg-bbl-surface border border-bbl-border text-bbl-text
                 rounded-bbl-sm px-2.5 py-1.5 text-xs cursor-pointer outline-none
                 focus:border-bbl-accent2 pr-7">
          <option value="en">English</option>
          <option value="es">Español</option>
        </select>
        <svg class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2
                    w-3 h-3 text-bbl-text-muted"
             viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 9l6 6 6-6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    `;

        return html`
      ${this._sectionHeader(this._t('settings.language.interface_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${this._row(
            this._t('settings.language.ui_language'),
            this._t('settings.language.ui_language_sub'),
            langSelect
        )}
      </div>
    `;
    }

    render() {
        const micIcon = html`
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor"
           stroke-width="1.7" stroke-linecap="round">
        <path d="M12 2a4 4 0 0 0-4 4v6a4 4 0 0 0 8 0V6a4 4 0 0 0-4-4z"/>
        <path d="M6 10v2a6 6 0 0 0 12 0v-2"/>
      </svg>`;

        const langIcon = html`
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor"
           stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2 5h10M7 2v3M11 5c0 4-2.5 7-5 9"/>
        <path d="M6 12c1 1.5 3 3 5 4M12 5l5 14M15.5 13h5"/>
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

          <nav class="flex items-center gap-1">
            ${this._navItem('audio', this._t('settings.nav.audio'), micIcon)}
            ${this._navItem('language', this._t('settings.nav.language'), langIcon)}
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
          ${this._section === 'audio' ? this._audioSection() : ''}
          ${this._section === 'language' ? this._languageSection() : ''}
        </div>

      </div>
    `;
    }
}
