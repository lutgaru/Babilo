/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { AIState } from '../types/babilo';

import './bbl-waveform';
import './bbl-avatar';

@customElement('bbl-stage')
export class BblStage extends withI18n(LitElement) {
  @property({ type: String })
  aiState: AIState = 'idle';

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 22px;
      padding: 32px 24px;
      flex: 1;
      overflow: hidden;
    }
    @media (max-height: 600px) {
      :host { gap: 14px; padding: 18px 20px; }
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  render() {
    return html`
      <div class="flex flex-col items-center gap-4 py-4">
        <bbl-avatar .state=${this.aiState}></bbl-avatar>
        <div class="flex flex-col items-center gap-0.5">
          <span class="text-[11px] uppercase tracking-[0.1em] text-[var(--bbl-text-faint)]">
            ${this._t('common.assistant')}
          </span>
          <span class="text-[15px] font-medium text-[var(--bbl-text)] min-h-[22px] text-center">
            ${this._t(`state.${this.aiState}` as any)}
          </span>
        </div>
        <bbl-waveform .state=${this.aiState}></bbl-waveform>
      </div>
    `;
  }
}