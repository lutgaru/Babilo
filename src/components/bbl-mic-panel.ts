/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { listAudioDevices } from '../invoke';

export class BblMicPanel extends LitElement {
  static properties = {
    devices:  { state: true },
    selected: { state: true },
  };

  constructor() {
    super();
    this.devices  = [];
    this.selected = '';
  }

  static styles = css`
    :host { display: block; }

    .row {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .row-label {
      display: flex;
      align-items: center;
      gap: 5px;
      font-size: 12px;
      color: var(--bbl-text-muted);
      white-space: nowrap;
    }
    .row-label svg { width: 14px; height: 14px; }

    .select-wrap {
      flex: 1;
      position: relative;
      display: flex;
      align-items: center;
    }

    select {
      width: 100%;
      appearance: none;
      -webkit-appearance: none;
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      border-radius: var(--bbl-radius-sm);
      padding: 7px 28px 7px 12px;
      font-size: 13px;
      color: var(--bbl-text);
      cursor: pointer;
      outline: none;
      transition: border-color 0.2s;
      font-family: inherit;
    }
    select:hover  { border-color: var(--bbl-border-md); }
    select:focus  { border-color: var(--bbl-accent2); }
    select option { background: var(--bbl-panel); color: var(--bbl-text); }

    .chevron {
      position: absolute;
      right: 10px;
      font-size: 11px;
      color: var(--bbl-text-muted);
      pointer-events: none;
    }

    .refresh {
      width: 32px;
      height: 32px;
      border-radius: var(--bbl-radius-sm);
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--bbl-text-muted);
      flex-shrink: 0;
      transition: background 0.15s, color 0.15s;
    }
    .refresh:hover { background: var(--bbl-btn-hover); color: var(--bbl-text); }
    .refresh svg   { width: 14px; height: 14px; }
  `;

  connectedCallback() {
    super.connectedCallback();
    this.load();
  }

  private async load() {
    try {
      this.devices = await listAudioDevices();
    } catch (err) {
      console.error('❌ Error cargando dispositivos:', err);
    }
  }

  get selectedDevice() { return this.selected || null; }

  render() {
    return html`
      <div class="row">
        <span class="row-label">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="5" y="1" width="6" height="8" rx="3"/>
            <path d="M2 7v1a6 6 0 0 0 12 0V7" stroke-linecap="round"/>
            <path d="M8 14v2M6 16h4" stroke-linecap="round"/>
          </svg>
          Micro
        </span>
        <div class="select-wrap">
          <select @change=${(e: Event) => { this.selected = (e.target as HTMLSelectElement).value; }}>
            <option value="">Predeterminado</option>
            ${this.devices.map(d => html`<option value=${d.name}>${d.name}</option>`)}
          </select>
          <span class="chevron">▾</span>
        </div>
        <button class="refresh" title="Refrescar" @click=${this.load}>
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2 8a6 6 0 1 0 1.5-3.9" stroke-linecap="round"/>
            <path d="M2 4v4h4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
      </div>
    `;
  }
}

customElements.define('bbl-mic-panel', BblMicPanel);