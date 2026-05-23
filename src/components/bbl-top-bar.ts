/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

@customElement('bbl-top-bar')
export class BblTopBar extends withI18n(LitElement) {

  @property({ type: String })
  status = 'Ready';

  @property({ type: Number })
  seconds = 0;

  @property({ type: String })
  modeName = '';

  @property({ type: Boolean })
  active = false;

  /** Mirrors whether bbl-settings is open — drives the button's active style */
  @property({ type: Boolean })
  settingsOpen = false;

  static styles = css`
    :host { display: block; }
    .brand-accent { letter-spacing: -0.025em; }
    .font-variant-numeric-tabular { font-variant-numeric: tabular-nums; }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  private get timeLabel(): string {
    const m = String(Math.floor(this.seconds / 60)).padStart(2, '0');
    const s = String(this.seconds % 60).padStart(2, '0');
    return `${m}:${s}`;
  }

  private _openSettings() {
    this.dispatchEvent(new CustomEvent('settings-open', { bubbles: true, composed: true }));
  }

  render() {
    return html`
      <header class="flex items-center justify-between py-3.5 px-5
                     border-b border-[var(--bbl-border)] bg-[var(--bbl-bg)]">

        <!-- Left: Brand + Mode name -->
        <div class="flex items-center gap-2 min-w-0">
          ${this.active
        ? html`<span class="w-2 h-2 rounded-full bg-red-500 animate-pulse flex-shrink-0"
                         title="${this._t('topbar.active_session')}"></span>`
        : html`<span class="w-2 h-2 rounded-full bg-[var(--bbl-muted)] flex-shrink-0"></span>`
      }
          <span class="text-sm font-medium text-[var(--bbl-text)] truncate">
            ${this.modeName
        ? this.modeName
        : html`babi<span class="brand-accent text-[var(--bbl-accent)]">lo</span>`
      }
          </span>
        </div>

        <!-- Center: Status + Timer -->
        <div class="flex items-center gap-3">
          <span class="text-xs text-[var(--bbl-muted)] capitalize">${this.status}</span>
          ${this.active
        ? html`<span class="text-xs font-variant-numeric-tabular text-[var(--bbl-text)]">
                     ${this.timeLabel}
                   </span>`
        : ''
      }
        </div>

        <!-- Right: Settings button -->
        <button
          @click=${this._openSettings}
          aria-label="${this._t('topbar.settings')}"
          aria-expanded="${this.settingsOpen}"
          class="
            w-8 h-8 rounded-full flex items-center justify-center
            transition-[background,color] duration-150
            ${this.settingsOpen
        ? 'bg-[var(--bbl-accent2)] text-white'
        : 'bg-[var(--bbl-btn-bg)] text-[var(--bbl-text-muted)] hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]'}
          ">
          <svg class="w-[15px] h-[15px]" viewBox="0 0 24 24" fill="none"
               stroke="currentColor" stroke-width="1.7" stroke-linecap="round"
               stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06
                     a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09
                     A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83
                     l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09
                     A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83
                     l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09
                     a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83
                     l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09
                     a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </button>

      </header>
    `;
  }
}