/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';

export class BblTopBar extends LitElement {
  static properties = {
    status:  {},
    seconds: {},
    active:  { type: Boolean },
  };

  constructor() {
    super();
    this.status  = 'Ready';
    this.seconds = 0;
    this.active  = false;
  }

  static styles = css`
    :host { display: block; }

    header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 14px 20px;
      background: var(--bbl-surface);
      border-bottom: 0.5px solid var(--bbl-border);
    }

    .brand {
      font-size: 18px;
      font-weight: 600;
      letter-spacing: 0.06em;
      color: var(--bbl-text);
    }
    .brand-accent { color: var(--bbl-accent); }

    .right {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .status {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .dot {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: var(--bbl-status-idle);
      transition: background 0.4s;
    }
    .dot.active { background: var(--bbl-status-live); }

    .label {
      font-size: 12px;
      color: var(--bbl-text-muted);
      min-width: 64px;
    }

    .timer {
      font-size: 12px;
      font-variant-numeric: tabular-nums;
      color: var(--bbl-text-muted);
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      border-radius: var(--bbl-radius-pill);
      padding: 4px 12px;
    }
  `;

  private get timeLabel() {
    const m = String(Math.floor(this.seconds / 60)).padStart(2, '0');
    const s = String(this.seconds % 60).padStart(2, '0');
    return `${m}:${s}`;
  }

  render() {
    return html`
      <header>
        <div class="brand">babi<span class="brand-accent">lo</span></div>
        <div class="right">
          <div class="status">
            <span class="dot ${this.active ? 'active' : ''}"></span>
            <span class="label">${this.status}</span>
          </div>
          <div class="timer">${this.timeLabel}</div>
        </div>
      </header>
    `;
  }
}

customElements.define('bbl-top-bar', BblTopBar);