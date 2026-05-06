/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property } from 'lit/decorators.js';
import './bbl-transcript';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { AIState } from '../types/babilo';


export class BblStage extends LitElement {
  // ── Reactive Properties ──
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

    /* ── Animations (impossible in Tailwind utilities) ── */
    @keyframes pulse-a {
      0%   { transform: scale(1);    opacity: 0.7; }
      100% { transform: scale(1.15); opacity: 0;   }
    }
    @keyframes pulse-b {
      0%   { transform: scale(1);    opacity: 0.5; }
      100% { transform: scale(1.12); opacity: 0;   }
    }
    @keyframes wave-idle {
      0%, 100% { height: 4px;  }
      50%       { height: 10px; }
    }
    @keyframes wave-live {
      0%, 100% { height: 5px;  }
      50%       { height: 26px; }
    }

    /* ── Pulsing rings (state-dependent animation) ── */
    .pulsing .ring-a { animation: pulse-a 1.6s ease-out infinite; }
    .pulsing .ring-b { animation: pulse-b 1.6s 0.3s ease-out infinite; }

    /* ── Waveform idle animations (nth-child impossible in Tailwind) ── */
    .bar:nth-child(1)  { animation: wave-idle 2.2s 0.0s ease-in-out infinite; }
    .bar:nth-child(2)  { animation: wave-idle 2.4s 0.2s ease-in-out infinite; }
    .bar:nth-child(3)  { animation: wave-idle 2.1s 0.1s ease-in-out infinite; }
    .bar:nth-child(4)  { animation: wave-idle 2.3s 0.3s ease-in-out infinite; }
    .bar:nth-child(13) { animation: wave-idle 2.2s 0.2s ease-in-out infinite; }
    .bar:nth-child(14) { animation: wave-idle 2.4s 0.0s ease-in-out infinite; }
    .bar:nth-child(15) { animation: wave-idle 2.1s 0.3s ease-in-out infinite; }
    .bar:nth-child(16) { animation: wave-idle 2.3s 0.1s ease-in-out infinite; }

    .bar.active:nth-child(5)  { animation: wave-idle 1.8s 0.0s ease-in-out infinite; }
    .bar.active:nth-child(6)  { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar.active:nth-child(7)  { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar.active:nth-child(8)  { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }
    .bar.active:nth-child(9)  { animation: wave-idle 1.8s 0.3s ease-in-out infinite; }
    .bar.active:nth-child(10) { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar.active:nth-child(11) { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar.active:nth-child(12) { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }

    /* ── Live state overrides ── */
    .waveform.live .bar.active { background: var(--bbl-accent); opacity: 1; }
    .waveform.live .bar.active:nth-child(5)  { animation: wave-live 0.90s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(6)  { animation: wave-live 0.80s 0.10s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(7)  { animation: wave-live 1.00s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(8)  { animation: wave-live 0.85s 0.20s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(9)  { animation: wave-live 0.95s 0.10s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(10) { animation: wave-live 0.80s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(11) { animation: wave-live 1.00s 0.15s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(12) { animation: wave-live 0.90s 0.05s ease-in-out infinite; }

    /* ── Responsive (media queries can't be done in Tailwind shadow DOM) ── */
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

  // ── Getters for state-derived classes ──
  get pulsing(): boolean {
    return this.aiState === 'listening' || this.aiState === 'speaking';
  }

  get live(): boolean {
    return this.aiState === 'listening';
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

        <!-- ring-c: filled innermost -->
        <div class="ring ring-c
                    absolute rounded-full pointer-events-none
                    w-[98px] h-[98px]
                    border border-[var(--bbl-ring-c)] bg-[var(--bbl-ring-c)]">
        </div>

        <!-- ring-b -->
        <div class="ring ring-b
                    absolute rounded-full pointer-events-none
                    w-32 h-32
                    border border-[var(--bbl-ring-b)]">
        </div>

        <!-- ring-a: outermost -->
        <div class="ring ring-a
                    absolute rounded-full pointer-events-none
                    w-40 h-40
                    border border-[var(--bbl-ring-a)]">
        </div>

        <!-- avatar circle -->
        <div class="avatar
                    relative z-[2]
                    w-[76px] h-[76px] rounded-full
                    bg-gradient-to-br from-[var(--bbl-accent2)] to-[var(--bbl-accent)]
                    flex items-center justify-center
                    text-[22px] font-semibold text-white
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

      <!-- Waveform -->
      <div class="waveform ${this.live ? 'live' : ''}
                  flex items-center gap-[3px] h-8"
           aria-hidden="true">
        <!-- outer-left (no active) -->
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <!-- active center bars -->
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <div class="bar active w-[3px] h-[5px] rounded-sm bg-[var(--bbl-accent2)] opacity-70"></div>
        <!-- outer-right (no active) -->
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
        <div class="bar w-[3px] h-[5px] rounded-sm bg-[var(--bbl-text-faint)] opacity-50"></div>
      </div>

      <!-- Transcript -->
      <bbl-transcript
        id="chat"
        .messages=${this.messages}>
      </bbl-transcript>
    `;
  }
}

customElements.define('bbl-stage', BblStage);