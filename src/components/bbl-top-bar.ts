/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

@customElement('bbl-top-bar')
export class BblTopBar extends LitElement {
  
  @property({ type: String })
  status = 'Ready';

  @property({ type: Number })
  seconds = 0;

  @property({ type: Boolean })
  active = false;
  
  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  static styles = css`
    :host { display: block; }
    /* Solo CSS personalizado que NO puede ser utility */
  `;

  private get timeLabel() {
    const m = String(Math.floor(this.seconds / 60)).padStart(2, '0');
    const s = String(this.seconds % 60).padStart(2, '0');
    return `${m}:${s}`;
  }

  render() {
    return html`
      <header class="flex items-center justify-between py-3.5 px-5 bg-bbl-surface border-b border-bbl-border">
        <div class="brand text-lg font-semibold tracking-wide text-bbl-text">
          babi<span class="brand-accent text-bbl-accent">lo</span>
        </div>
        <div class="right flex items-center gap-3">
          <div class="status flex items-center gap-1.5">
            <span class="dot w-1.75 h-1.75 rounded-full bg-bbl-status-idle transition-colors duration-400 
                        ${this.active ? 'active bg-bbl-status-live' : ''}"></span>
            <span class="label text-xs text-bbl-text-muted min-w-16">${this.status}</span>
          </div>
          <div class="timer text-xs font-variant-numeric-tabular text-bbl-text-muted 
                      bg-bbl-btn-bg border border-bbl-border rounded-bbl-pill px-3 py-1">
            ${this.timeLabel}
          </div>
        </div>
      </header>
    `;
  }
}