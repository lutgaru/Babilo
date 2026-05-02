/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';

export class BblControls extends LitElement {
  static properties = {
    recording: { type: Boolean },
  };

  constructor() {
    super();
    this.recording = false;
  }

  static styles = css`
    :host { display: block; }

    footer {
      background: var(--bbl-surface);
      border-top: 0.5px solid var(--bbl-border);
      padding: 16px 20px 20px;
      display: flex;
      flex-direction: column;
      gap: 14px;
    }

    /* ── Fila de acciones ─────────────────────────────────────────── */
    .action-row {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 14px;
    }

    /* Botón circular secundario (mute, volumen, ajustes) */
    .ctrl-btn {
      width: 48px;
      height: 48px;
      border-radius: 50%;
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--bbl-text-muted);
      transition: background 0.15s, color 0.15s, transform 0.1s;
    }
    .ctrl-btn:hover  { background: var(--bbl-btn-hover); color: var(--bbl-text); }
    .ctrl-btn:active { background: var(--bbl-btn-active); transform: scale(0.94); }
    .ctrl-btn svg    { width: 18px; height: 18px; }

    /* Botón principal de micrófono */
    .mic-main {
      width: 68px;
      height: 68px;
      border-radius: 50%;
      background: var(--bbl-accent2);
      border: none;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #fff;
      transition: background 0.2s, transform 0.1s, box-shadow 0.2s;
    }
    .mic-main:hover  { background: #7352d4; }
    .mic-main:active { transform: scale(0.94); }
    .mic-main svg    { width: 22px; height: 22px; }

    .mic-main.recording {
      background: var(--bbl-accent);
      animation: mic-pulse 2s ease-in-out infinite;
    }
    @keyframes mic-pulse {
      0%, 100% { box-shadow: 0 0 0 4px var(--bbl-accent-ring); }
      50%       { box-shadow: 0 0 0 10px rgba(224, 68, 106, 0.08); }
    }

    /* Botón colgar */
    .end-btn {
      width: 48px;
      height: 48px;
      border-radius: 50%;
      background: var(--bbl-accent-dim);
      border: 0.5px solid var(--bbl-accent-ring);
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--bbl-accent);
      transition: background 0.15s, transform 0.1s;
    }
    .end-btn:hover  { background: rgba(224, 68, 106, 0.25); }
    .end-btn:active { transform: scale(0.94); }
    .end-btn svg    { width: 18px; height: 18px; }

    /* ── Input de texto ───────────────────────────────────────────── */
    .input-row {
      display: flex;
      gap: 8px;
      align-items: center;
    }

    input {
      flex: 1;
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      border-radius: var(--bbl-radius-pill);
      padding: 9px 18px;
      font-size: 14px;
      color: var(--bbl-text);
      outline: none;
      transition: border-color 0.2s;
    }
    input::placeholder { color: var(--bbl-text-faint); }
    input:focus        { border-color: var(--bbl-accent2); }

    .send-btn {
      width: 38px;
      height: 38px;
      border-radius: 50%;
      background: var(--bbl-accent2);
      border: none;
      display: flex;
      align-items: center;
      justify-content: center;
      color: #fff;
      flex-shrink: 0;
      transition: background 0.15s, transform 0.1s;
    }
    .send-btn:hover  { background: #7352d4; }
    .send-btn:active { transform: scale(0.94); }
    .send-btn svg    { width: 14px; height: 14px; }
  `;

  private _onMicClick() {
    this.dispatchEvent(new CustomEvent('mic-toggle', { bubbles: true, composed: true }));
  }

  private _onSubmit(e: Event) {
    e.preventDefault();
    const input = this.shadowRoot!.querySelector('input') as HTMLInputElement;
    this.dispatchEvent(new CustomEvent('prompt-submit', {
      detail: input.value,
      bubbles: true,
      composed: true,
    }));
  }

  render() {
    return html`
      <footer>
        <slot name="mic-panel"></slot>

        <div class="action-row">
          <button class="ctrl-btn" title="Silenciar">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
              <path d="M12 2a4 4 0 0 0-4 4v6a4 4 0 0 0 8 0V6a4 4 0 0 0-4-4z"/>
              <path d="M6 10v2a6 6 0 0 0 12 0v-2" stroke-linecap="round"/>
              <path d="M12 19v3M9 22h6" stroke-linecap="round"/>
            </svg>
          </button>

          <button class="ctrl-btn" title="Volumen">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
              <path d="M11 5L6 9H2v6h4l5 4V5z" stroke-linejoin="round"/>
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14" stroke-linecap="round"/>
            </svg>
          </button>

          <button class="mic-main ${this.recording ? 'recording' : ''}"
                  title="Grabar / Detener"
                  @click=${this._onMicClick}>
            <svg viewBox="0 0 24 24" fill="currentColor">
              <rect x="9" y="2" width="6" height="12" rx="3"/>
              <path d="M5 10v2a7 7 0 0 0 14 0v-2"
                    stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round"/>
              <path d="M12 19v3M9 22h6"
                    stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round"/>
            </svg>
          </button>

          <button class="ctrl-btn" title="Configuración">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
              <circle cx="12" cy="12" r="3"/>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
                    stroke-linecap="round"/>
            </svg>
          </button>

          <button class="end-btn" title="Colgar">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
              <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"
                    stroke-linecap="round"/>
              <line x1="4" y1="4" x2="20" y2="20" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <form class="input-row" @submit=${this._onSubmit}>
          <input id="greet-input"
                 type="text"
                 placeholder="Write a prompt to test the AI response..."
                 autocomplete="off"/>
          <button type="submit" class="send-btn" title="Enviar">
            <svg viewBox="0 0 16 16" fill="currentColor">
              <path d="M2 8l12-6-6 12-.5-5.5L2 8z"/>
            </svg>
          </button>
        </form>
      </footer>
    `;
  }
}

customElements.define('bbl-controls', BblControls);