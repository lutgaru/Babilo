/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { withI18n } from '../i18n';
import { state, customElement } from 'lit/decorators.js';
import { ModeFileInfo, SessionInfo } from '../types/babilo';
import { listModes, startSession } from '../invoke';

// Directory where .babilo.json files live — adjust according to your Tauri structure

@customElement('bbl-config-list')
export class BblConfigList extends withI18n(LitElement) {
  @state() private modes: ModeFileInfo[] = [];
  @state() private loading = true;
  @state() private starting: string | null = null; // path of mode being started
  @state() private error: string | null = null;

  static styles = css`:host { display: block; }`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    this.loadModes();
  }

  private async loadModes() {
    try {
      this.modes = await listModes();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  private async selectMode(mode: ModeFileInfo) {
    if (this.starting) return;
    this.starting = mode.path;
    try {
      console.log(`Iniciando modo: ${mode.path}`);
      const info: SessionInfo = await startSession(mode.path);
      this.dispatchEvent(new CustomEvent<{ info: SessionInfo; path: string }>('session-started', {
        detail: { info, path: mode.path },
        bubbles: true,
        composed: true,
      }));
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      this.starting = null;
    }
  }

  private _renderCapIcons(mode: ModeFileInfo) {
    const icons = [];
    if (mode.caps.accepts_audio) icons.push(html`<span title="${this._t('config.cap.audio')}">🎙</span>`);
    if (mode.caps.accepts_text) icons.push(html`<span title="${this._t('config.cap.text')}">⌨️</span>`);
    if (mode.caps.llm_initiates) icons.push(html`<span title="${this._t('config.cap.llm_first')}">🤖</span>`);
    return icons;
  }

  render() {
    if (this.loading) return html`
      <div class="flex items-center justify-center h-screen bg-[var(--bbl-bg)]">
        <p class="text-[var(--bbl-muted)] text-sm">${this._t('common.loading')}</p>
      </div>
    `;

    return html`
      <div class="flex flex-col h-screen bg-[var(--bbl-bg)]">

        <!-- Header -->
        <header class="px-6 pt-8 pb-4">
          <h1 class="text-2xl font-semibold text-[var(--bbl-text)]">${this._t('config.title')}</h1>
          <p class="text-sm text-[var(--bbl-muted)] mt-1">${this._t('config.subtitle')}</p>
        </header>

        <!-- Error -->
        ${this.error ? html`
          <div class="mx-6 mb-4 px-4 py-3 rounded-lg bg-red-500/10 text-red-400 text-sm">
            ${this._t('common.error')}: ${this.error}
          </div>
        ` : ''}

        <!-- Lista de configs -->
        <ul class="flex-1 overflow-y-auto px-6 space-y-3 pb-8">
          ${this.modes.length === 0 ? html`
            <li class="text-[var(--bbl-muted)] text-sm py-8 text-center">
              ${this._t('config.no_modes')}
            </li>
          ` : this.modes.map(mode => html`
            <li>
              <button
                @click=${() => this.selectMode(mode)}
                ?disabled=${!!this.starting}
                class="w-full text-left px-5 py-4 rounded-2xl
                       bg-[var(--bbl-surface)] border border-[var(--bbl-border)]
                       hover:border-[var(--bbl-accent)] hover:bg-[var(--bbl-surface-hover)]
                       active:scale-[0.98] transition-all duration-150
                       disabled:opacity-50 disabled:cursor-not-allowed
                       flex items-center gap-4">

                <!-- Icono de modo -->
                <div class="w-12 h-12 rounded-xl bg-[var(--bbl-accent)]/10
                            flex items-center justify-center text-2xl flex-shrink-0">
                  ${mode.caps.llm_initiates
        ? html`<span title="${this._t('config.cap.llm_first')}">🎧</span>`
        : html`<span title="${this._t('config.cap.text')}">💬</span>`}
                </div>

                <!-- Info -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-[var(--bbl-text)] truncate">
                      ${mode.name}
                    </span>
                    <span class="flex gap-1 text-xs opacity-60">
                      ${this._renderCapIcons(mode)}
                    </span>
                  </div>
                  ${mode.description ? html`
                    <p class="text-xs text-[var(--bbl-muted)] mt-0.5 line-clamp-2">
                      ${mode.description}
                    </p>
                  ` : ''}
                </div>

                <!-- Estado -->
                <div class="flex-shrink-0 text-[var(--bbl-muted)]">
                  ${this.starting === mode.path
        ? html`<span class="text-xs animate-pulse">${this._t('config.starting')}</span>`
        : html`<span class="text-lg">›</span>`
      }
                </div>
              </button>
            </li>
          `)}
        </ul>

      </div>
    `;
  }
}