/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import './bbl-transcript';

export type AIState = 'idle' | 'listening' | 'processing' | 'speaking';

const STATE_LABELS: Record<AIState, string> = {
  idle:       'Waiting...',
  listening:  'Listening...',
  processing: 'Processing...',
  speaking:   'Speaking...',
};

export class BblStage extends LitElement {
  static properties = {
    aiState:  {},
    response: {},
  };

  constructor() {
    super();
    this.aiState  = 'idle' as AIState;
    this.response = '';
  }

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

    /* ── Avatar + anillos ─────────────────────────────────────────── */
    .avatar-wrap {
      position: relative;
      width: 160px;
      height: 160px;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .ring {
      position: absolute;
      border-radius: 50%;
      border: 1px solid;
      pointer-events: none;
    }
    .ring-a { width: 160px; height: 160px; border-color: var(--bbl-ring-a); }
    .ring-b { width: 128px; height: 128px; border-color: var(--bbl-ring-b); }
    .ring-c { width:  98px; height:  98px; border-color: var(--bbl-ring-c); background: var(--bbl-ring-c); }

    .avatar {
      width: 76px;
      height: 76px;
      border-radius: 50%;
      background: linear-gradient(135deg, var(--bbl-accent2) 0%, var(--bbl-accent) 100%);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      font-weight: 600;
      color: #fff;
      position: relative;
      z-index: 2;
      border: 1.5px solid rgba(255, 255, 255, 0.15);
      letter-spacing: 0.04em;
    }

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

    /* ── Info de estado ───────────────────────────────────────────── */
    .ai-info {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
    }
    .ai-label {
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: var(--bbl-text-faint);
    }
    .ai-state {
      font-size: 15px;
      font-weight: 500;
      color: var(--bbl-text);
      min-height: 22px;
    }

    /* ── Visualizador de ondas ────────────────────────────────────── */
    .waveform {
      display: flex;
      align-items: center;
      gap: 3px;
      height: 32px;
    }

    .bar {
      width: 3px;
      height: 5px;
      border-radius: 2px;
      background: var(--bbl-text-faint);
      opacity: 0.5;
    }
    .bar.active {
      background: var(--bbl-accent2);
      opacity: 0.7;
    }

    @keyframes wave-idle { 0%, 100% { height: 4px;  } 50% { height: 10px; } }
    @keyframes wave-live { 0%, 100% { height: 5px;  } 50% { height: 26px; } }

    /* Barras inactivas (decorativas) */
    .bar:nth-child(1)  { animation: wave-idle 2.2s 0.0s ease-in-out infinite; }
    .bar:nth-child(2)  { animation: wave-idle 2.4s 0.2s ease-in-out infinite; }
    .bar:nth-child(3)  { animation: wave-idle 2.1s 0.1s ease-in-out infinite; }
    .bar:nth-child(4)  { animation: wave-idle 2.3s 0.3s ease-in-out infinite; }
    .bar:nth-child(13) { animation: wave-idle 2.2s 0.2s ease-in-out infinite; }
    .bar:nth-child(14) { animation: wave-idle 2.4s 0.0s ease-in-out infinite; }
    .bar:nth-child(15) { animation: wave-idle 2.1s 0.3s ease-in-out infinite; }
    .bar:nth-child(16) { animation: wave-idle 2.3s 0.1s ease-in-out infinite; }

    /* Barras activas — idle */
    .bar.active:nth-child(5)  { animation: wave-idle 1.8s 0.0s ease-in-out infinite; }
    .bar.active:nth-child(6)  { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar.active:nth-child(7)  { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar.active:nth-child(8)  { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }
    .bar.active:nth-child(9)  { animation: wave-idle 1.8s 0.3s ease-in-out infinite; }
    .bar.active:nth-child(10) { animation: wave-idle 1.6s 0.1s ease-in-out infinite; }
    .bar.active:nth-child(11) { animation: wave-idle 1.9s 0.2s ease-in-out infinite; }
    .bar.active:nth-child(12) { animation: wave-idle 1.7s 0.0s ease-in-out infinite; }

    /* Barras activas — live */
    .waveform.live .bar.active { background: var(--bbl-accent); opacity: 1; }
    .waveform.live .bar.active:nth-child(5)  { animation: wave-live 0.90s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(6)  { animation: wave-live 0.80s 0.10s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(7)  { animation: wave-live 1.00s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(8)  { animation: wave-live 0.85s 0.20s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(9)  { animation: wave-live 0.95s 0.10s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(10) { animation: wave-live 0.80s 0.00s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(11) { animation: wave-live 1.00s 0.15s ease-in-out infinite; }
    .waveform.live .bar.active:nth-child(12) { animation: wave-live 0.90s 0.05s ease-in-out infinite; }

    /* ── Responsive ───────────────────────────────────────────────── */
    @media (max-height: 600px) {
      :host      { gap: 14px; padding: 18px 20px; }
      .avatar-wrap { width: 120px; height: 120px; }
      .ring-a    { width: 120px; height: 120px; }
      .ring-b    { width:  96px; height:  96px; }
      .ring-c    { width:  74px; height:  74px; }
      .avatar    { width:  60px; height:  60px; font-size: 18px; }
    }
  `;

  private get pulsing() { return this.aiState === 'listening' || this.aiState === 'speaking'; }
  private get live()    { return this.aiState === 'listening'; }

  render() {
    return html`
      <div class="avatar-wrap ${this.pulsing ? 'pulsing' : ''}">
        <div class="ring ring-c"></div>
        <div class="ring ring-b"></div>
        <div class="ring ring-a"></div>
        <div class="avatar">AI</div>
      </div>

      <div class="ai-info">
        <span class="ai-label">Assistant</span>
        <span class="ai-state">${STATE_LABELS[this.aiState as AIState]}</span>
      </div>

      <div class="waveform ${this.live ? 'live' : ''}" aria-hidden="true">
        <div class="bar"></div><div class="bar"></div>
        <div class="bar"></div><div class="bar"></div>
        <div class="bar active"></div><div class="bar active"></div>
        <div class="bar active"></div><div class="bar active"></div>
        <div class="bar active"></div><div class="bar active"></div>
        <div class="bar active"></div><div class="bar active"></div>
        <div class="bar"></div><div class="bar"></div>
        <div class="bar"></div><div class="bar"></div>
      </div>

      <bbl-transcript .text=${this.response}></bbl-transcript>
    `;
  }
}

customElements.define('bbl-stage', BblStage);