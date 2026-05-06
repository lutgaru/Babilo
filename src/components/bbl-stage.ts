/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property } from 'lit/decorators.js';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { AIState, TranscriptMessage } from '../types/babilo';

import './bbl-transcript';
import './bbl-waveform';
import './bbl-avatar'; // ← Nuevo import

export class BblStage extends LitElement {
  @property({ type: String })
  aiState: AIState = 'idle';

  @property({ type: Array })
  messages: Array<TranscriptMessage> = [];

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

    /* ── Responsive ── */
    @media (max-height: 600px) {
      :host { gap: 14px; padding: 18px 20px; }
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  render() {
    const STATE_LABELS: Record<AIState, string> = {
      idle: 'Waiting...',
      listening: 'Listening...',
      processing: 'Processing...',
      speaking: 'Speaking...',
      thinking: 'Thinking...'
    };

    return html`
<div class="grid grid-cols-2 gap-2 w-full items-start justify-center">
  
  <!-- Columna izquierda: Avatar + Label + Waveform -->
  <div class="flex flex-col items-center gap-4 py-4">
    
    <!-- Avatar (extracted component) -->
    <bbl-avatar .state=${this.aiState}></bbl-avatar>

    <!-- AI label + state (centrado horizontalmente) -->
    <div class="flex flex-col items-center gap-0.5">
      <span class="text-[11px] uppercase tracking-[0.1em] text-[var(--bbl-text-faint)]">
        Assistant
      </span>
      <span class="text-[15px] font-medium text-[var(--bbl-text)] min-h-[22px] text-center">
        ${STATE_LABELS[this.aiState]}
      </span>
    </div>

    <!-- Waveform (extracted component) -->
    <bbl-waveform .state=${this.aiState}></bbl-waveform>
  </div>

  <!-- Columna derecha: Transcript (ocupa altura completa) -->
  <div class="flex flex-col min-h-[200px] ">
    <bbl-transcript 
      id="chat" 
      .messages=${this.messages}
      class="w-full h-full">
    </bbl-transcript>
  </div>

</div>
    `;
  }
}

customElements.define('bbl-stage', BblStage);