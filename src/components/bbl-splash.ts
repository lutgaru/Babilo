/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';

@customElement('bbl-splash')
export class BblSplash extends withI18n(LitElement) {
  @property({ type: String }) message = 'Initializing language engines...';
  @property({ type: String }) errorMessage = '';

  static styles = css`
  :host {
    display: block;
  }
  .loading-bar-track {
    width: 100%;
    height: 3px;
    border-radius: 999px;
    background: var(--bbl-border);
    overflow: hidden;
    position: relative;
  }
  .loading-bar-fill {
    position: absolute;
    inset: 0;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--bbl-accent2) 0%,
      var(--bbl-accent) 50%,
      var(--bbl-accent2) 100%
    );
    background-size: 200% auto;
    animation: shimmer 4s linear infinite;
  }
  .brand-glow {
    animation: pulse-glow 2s infinite ease-in-out;
  }
  @keyframes shimmer {
    0%   { background-position: -200% center; }
    100% { background-position:  200% center; }
  }
  @keyframes pulse-glow {
    0%, 100% { opacity: 0.6; transform: scale(0.98); }
    50%       { opacity: 1;   transform: scale(1.02); }
  }
`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  render() {
    return html`
      <div class="fixed inset-0 bg-[#111216] text-slate-200 flex flex-col items-center justify-center font-sans select-none">
        
        <div class=" flex flex-col items-center mb-12">
          <div class="text-5xl font-black tracking-wider text-transparent bg-clip-text bg-gradient-to-r from-blue-500 to-indigo-400">
            ${this._t('splash.brand')}
          </div>
          <div class="text-xs uppercase tracking-[0.3em] text-slate-500 mt-2 font-semibold">
            ${this._t('splash.tagline')}
          </div>
        </div>

        <div class="flex flex-col items-center min-h-[80px]">
          ${this.errorMessage
        ? html`
                <div class="flex flex-col items-center max-w-md text-center px-6">
                  <div class="text-red-400 bg-red-950/40 border border-red-900/50 rounded-lg p-4 text-sm font-mono mb-2">
                    ⚠️ ${this.errorMessage}
                  </div>
                  <p class="text-xs text-slate-500">${this._t('splash.error_hint')}</p>
                </div>
              `
        : html`
                <div class="loading-bar-track w-64 mb-4">
                    <div class="loading-bar-fill"></div>
                </div>
                <div class="text-sm font-medium text-slate-400 font-mono tracking-wide">
                  ${this.message === 'Initializing language engines...'
            ? this._t('splash.init')
            : this.message === 'Synchronizing Vulkan environments...'
              ? this._t('splash.sync_vulkan')
              : this.message === 'Instantiating network weights in VRAM...'
                ? this._t('splash.load_vram')
                : this.message}
                </div>
              `
      }
        </div>

        <div class="absolute bottom-6 text-[10px] font-mono text-slate-600 tracking-tight">
          ${this._t('splash.license')}
        </div>
      </div>
    `;
  }
}
