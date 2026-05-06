/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { listAudioDevices } from '../invoke';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

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

    /* ── Impossible in Tailwind ── */

    select { appearance: none; -webkit-appearance: none; }
    select:hover  { border-color: var(--bbl-border-md); }
    select:focus  { border-color: var(--bbl-accent2); }
    select option { background: var(--bbl-panel); color: var(--bbl-text); }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
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
      <div class="flex items-center gap-2">

        <!-- Label -->
        <span class="
          flex items-center gap-[5px]
          text-[12px] text-[var(--bbl-text-muted)]
          whitespace-nowrap
        ">
          <svg class="w-[14px] h-[14px]" viewBox="0 0 16 16"
               fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="5" y="1" width="6" height="8" rx="3"/>
            <path d="M2 7v1a6 6 0 0 0 12 0V7" stroke-linecap="round"/>
            <path d="M8 14v2M6 16h4" stroke-linecap="round"/>
          </svg>
          Micro
        </span>

        <!-- Select wrapper -->
        <div class="relative flex flex-1 items-center">
          <select
            class="
              w-full
              bg-[var(--bbl-btn-bg)] border border-[0.5px] border-[var(--bbl-border)]
              rounded-[var(--bbl-radius-sm)]
              py-[7px] pl-3 pr-7
              text-[13px] text-[var(--bbl-text)]
              font-[inherit] cursor-pointer outline-none
              transition-[border-color] duration-200
            "
            @change=${(e: Event) => { this.selected = (e.target as HTMLSelectElement).value; }}>
            <option value="">Default</option>
            ${this.devices.map((d: any) => html`
              <option value=${d.name}>${d.name}</option>
            `)}
          </select>

          <!-- Custom chevron -->
          <span class="
            absolute right-[10px]
            text-[11px] text-[var(--bbl-text-muted)]
            pointer-events-none
          ">▾</span>
        </div>

        <!-- Refresh button -->
        <button
          title="Refrescar"
          class="
            w-8 h-8 flex-shrink-0
            rounded-[var(--bbl-radius-sm)]
            bg-[var(--bbl-btn-bg)] border border-[0.5px] border-[var(--bbl-border)]
            flex items-center justify-center
            text-[var(--bbl-text-muted)]
            transition-[background,color] duration-150
            hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]
            active:scale-[0.94]
          "
          @click=${this.load}>
          <svg class="w-[14px] h-[14px]" viewBox="0 0 16 16"
               fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M2 8a6 6 0 1 0 1.5-3.9" stroke-linecap="round"/>
            <path d="M2 4v4h4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>

      </div>
    `;
  }
}

customElements.define('bbl-mic-panel', BblMicPanel);