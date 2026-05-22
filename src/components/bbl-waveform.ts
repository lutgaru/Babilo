/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property } from 'lit/decorators.js';
import { customElement } from 'lit/decorators.js';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { AIState } from '../types/babilo';

@customElement('bbl-waveform')
export class BblWaveform extends LitElement {
  @property({ type: String })
  state: AIState = 'idle';

  // ── Computed helpers ──
  private get _isLive(): boolean {
    return this.state === 'listening';
  }

  private get _isActive(): boolean {
    return this.state === 'listening' || this.state === 'speaking';
  }

  static styles = css`
    :host {
      display: block;
    }

    /* ── Keyframes ── */
    @keyframes wave-idle {
      0%, 100% { height: 4px;  }
      50%       { height: 10px; }
    }
    @keyframes wave-live {
      0%, 100% { height: 5px;  }
      50%       { height: 26px; }
    }

    /* ── Base bar styles ── */
    .bar {
      width: 3px;
      height: 5px;
      border-radius: 2px;
      background: var(--bbl-text-faint);
      opacity: 0.5;
      transition: background 0.2s, opacity 0.2s;
    }

    /* Active bars: se aplican cuando data-active="true" */
    .bar[data-active="true"] {
      background: var(--bbl-accent2);
      opacity: 0.7;
    }

    /* ── Idle animations (aplican a todas las barras) ── */
    .bar:nth-child(1)  { animation: wave-idle 2.2s 0.0s ease-in-out infinite; }
    .bar:nth-child(2)  { animation: wave-idle 2.4s 0.2s ease-in-out infinite; }
    .bar:nth-child(3)  { animation: wave-idle 2.1s 0.1s ease-in-out infinite; }
    .bar:nth-child(4)  { animation: wave-idle 2.3s 0.3s ease-in-out infinite; }
    .bar:nth-child(13) { animation: wave-idle 2.2s 0.2s ease-in-out infinite; }
    .bar:nth-child(14) { animation: wave-idle 2.4s 0.0s ease-in-out infinite; }
    .bar:nth-child(15) { animation: wave-idle 2.1s 0.3s ease-in-out infinite; }
    .bar:nth-child(16) { animation: wave-idle 2.3s 0.1s ease-in-out infinite; }

    .bar[data-active="true"]:nth-child(5)  { animation: wave-idle 1.8s 0.0s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(6)  { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(7)  { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(8)  { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(9)  { animation: wave-idle 1.8s 0.3s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(10) { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(11) { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar[data-active="true"]:nth-child(12) { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }

    /* ── LIVE STATE OVERRIDES (solo cuando data-live="true") ── */
    .bar[data-live="true"][data-active="true"] {
      background: var(--bbl-accent);
      opacity: 1;
    }
    .bar[data-live="true"][data-active="true"]:nth-child(5)  { animation: wave-live 0.90s 0.00s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(6)  { animation: wave-live 0.80s 0.10s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(7)  { animation: wave-live 1.00s 0.00s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(8)  { animation: wave-live 0.85s 0.20s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(9)  { animation: wave-live 0.95s 0.10s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(10) { animation: wave-live 0.80s 0.00s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(11) { animation: wave-live 1.00s 0.15s ease-in-out infinite; }
    .bar[data-live="true"][data-active="true"]:nth-child(12) { animation: wave-live 0.90s 0.05s ease-in-out infinite; }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  render() {
    return html`
      <div class="flex items-center gap-[3px] h-8" aria-hidden="true">
        <!-- Outer-left bars (inactive) -->
        ${this._renderBar(false)}
        ${this._renderBar(false)}
        ${this._renderBar(false)}
        ${this._renderBar(false)}
        
        <!-- Center bars (active) -->
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        ${this._renderBar(true)}
        
        <!-- Outer-right bars (inactive) -->
        ${this._renderBar(false)}
        ${this._renderBar(false)}
        ${this._renderBar(false)}
        ${this._renderBar(false)}
      </div>
    `;
  }

  private _renderBar(isActiveBar: boolean) {
    const shouldBeActive = isActiveBar && this._isActive;

    return html`
      <div 
        class="bar"
        data-active=${shouldBeActive}
        data-live=${this._isLive && shouldBeActive}>
      </div>
    `;
  }
}