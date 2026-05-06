/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property } from 'lit/decorators.js';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { AIState } from '../types/babilo';

import './bbl-transcript';
import './bbl-waveform'; // ← Nuevo import

export class BblStage extends LitElement {
  @property({ type: String })
  aiState: AIState = 'idle';

  @property({ type: Array })
  messages: Array<{ role: 'user' | 'ai'; content: string; timestamp?: number }> = [];

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

    /* ── Avatar ring animations (only what's unique to avatar) ── */
    @keyframes pulse-a {
      0%   { transform: scale(1);    opacity: 0.7; }
      100% { transform: scale(1.15); opacity: 0;   }
    }
    @keyframes pulse-b {
      0%   { transform: scale(1);    opacity: 0.5; }
      100% { transform: scale(1.12); opacity: 0;   }
    }
    .pulsing .ring-a { animation: pulse-a 1.6s ease-out infinite; }
    .pulsing .ring-b { animation: pulse-b 1.6s 0.3s ease-out infinite; }

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

  private get pulsing(): boolean {
    return this.aiState === 'listening' || this.aiState === 'speaking';
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
      <!-- Avatar + rings -->
      <div class="avatar-wrap ${this.pulsing ? 'pulsing' : ''}
                  relative w-40 h-40 flex items-center justify-center">
        <div class="ring ring-c absolute rounded-full pointer-events-none
                    w-[98px] h-[98px] border border-[var(--bbl-ring-c)] bg-[var(--bbl-ring-c)]"></div>
        <div class="ring ring-b absolute rounded-full pointer-events-none
                    w-32 h-32 border border-[var(--bbl-ring-b)]"></div>
        <div class="ring ring-a absolute rounded-full pointer-events-none
                    w-40 h-40 border border-[var(--bbl-ring-a)]"></div>
        <div class="avatar relative z-[2] w-[76px] h-[76px] rounded-full
                    bg-gradient-to-br from-[var(--bbl-accent2)] to-[var(--bbl-accent)]
                    flex items-center justify-center text-[22px] font-semibold text-white
                    border border-white/15 tracking-[0.04em]">
          AI
        </div>
      </div>

      <!-- AI label + state -->
      <div class="flex flex-col items-center gap-0.5">
        <span class="text-[11px] uppercase tracking-[0.1em] text-[var(--bbl-text-faint)]">
          Assistant
        </span>
        <span class="text-[15px] font-medium text-[var(--bbl-text)] min-h-[22px]">
          ${STATE_LABELS[this.aiState]}
        </span>
      </div>

      <!-- Waveform con atributos (¡más limpio!) -->
      <bbl-waveform .state=${this.aiState}></bbl-waveform>

      <!-- Transcript (sin cambios) -->
      <bbl-transcript id="chat" .messages=${this.messages}></bbl-transcript>
    `;
  }
}

customElements.define('bbl-stage', BblStage);