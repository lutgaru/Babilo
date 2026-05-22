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

  static styles = css`
    :host { display: block; }
    /* Custom CSS that can't be expressed with Tailwind utilities */
    .brand-accent { letter-spacing: -0.025em; }
    .font-variant-numeric-tabular { font-variant-numeric: tabular-nums; }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  private get timeLabel(): string {
    const m = String(Math.floor(this.seconds / 60)).padStart(2, '0');
    const s = String(this.seconds % 60).padStart(2, '0');
    return `${m}:${s}`;
  }

  private hangUp(): void {
    this.dispatchEvent(new CustomEvent('hang-up', {
      bubbles: true,
      composed: true
    }));
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

        <!-- Right: Hang-up button (only when session is active) -->
        ${this.modeName
        ? html`
            <button
              @click=${this.hangUp}
              class="w-8 h-8 rounded-full bg-red-500/10 hover:bg-red-500 
                     text-red-400 hover:text-white transition-all duration-150 
                     flex items-center justify-center text-sm"
              title="${this._t('topbar.end_session')}">
              ✕
            </button>
          `
        : html`<div class="w-8"></div>`
      }
      </header>
    `;
  }
}