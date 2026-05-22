/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

@customElement('bbl-controls')
export class BblControls extends withI18n(LitElement) {
  @property({ type: Boolean }) recording = false;

  /** Mirrors the transcript panel state — drives the icon's active style */
  @property({ type: Boolean }) transcriptOpen = true;

  static styles = css`
    :host { display: block; }

    @keyframes mic-pulse {
      0%, 100% { box-shadow: 0 0 0 4px var(--bbl-accent-ring); }
      50%       { box-shadow: 0 0 0 10px rgba(224, 68, 106, 0.08); }
    }
    .mic-main.recording {
      background: var(--bbl-accent);
      animation: mic-pulse 2s ease-in-out infinite;
    }

    input:focus { border-color: var(--bbl-accent2); }
    input::placeholder { color: var(--bbl-text-faint); }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  private _onMicClick() {
    this.dispatchEvent(new CustomEvent('mic-toggle', { bubbles: true, composed: true }));
  }

  private _onTranscriptToggle() {
    this.dispatchEvent(new CustomEvent('transcript-toggle', { bubbles: true, composed: true }));
  }

  render() {
    return html`
      <footer class="
        bg-[var(--bbl-surface)]
        border-t-[0.5px] border-[var(--bbl-border)]
        pt-4 pb-5 px-5
        flex flex-col gap-[14px]
      ">

        <slot name="mic-panel"></slot>

        <!-- Action row -->
        <div class="flex items-center justify-center gap-[14px]">

          <!-- Mute button -->
          <button
            title="${this._t('controls.mute')}"
            class="
              w-12 h-12 rounded-full
              bg-[var(--bbl-btn-bg)] border-[0.5px] border-[var(--bbl-border)]
              flex items-center justify-center
              text-[var(--bbl-text-muted)]
              transition-[background,color] duration-150
              hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]
              active:bg-[var(--bbl-btn-active)] active:scale-[0.94]
            ">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.6">
              <path d="M12 2a4 4 0 0 0-4 4v6a4 4 0 0 0 8 0V6a4 4 0 0 0-4-4z"/>
              <path d="M6 10v2a6 6 0 0 0 12 0v-2" stroke-linecap="round"/>
              <path d="M12 19v3M9 22h6" stroke-linecap="round"/>
            </svg>
          </button>

          <!-- Volume button -->
          <button
            title="${this._t('controls.volume')}"
            class="
              w-12 h-12 rounded-full
              bg-[var(--bbl-btn-bg)] border-[0.5px] border-[var(--bbl-border)]
              flex items-center justify-center
              text-[var(--bbl-text-muted)]
              transition-[background,color] duration-150
              hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]
              active:bg-[var(--bbl-btn-active)] active:scale-[0.94]
            ">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.6">
              <path d="M11 5L6 9H2v6h4l5 4V5z" stroke-linejoin="round"/>
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14"
                    stroke-linecap="round"/>
            </svg>
          </button>

          <!-- Main mic button -->
          <button
            title="${this._t('controls.record_toggle')}"
            class="
              mic-main
              w-[68px] h-[68px] rounded-full
              bg-[var(--bbl-accent2)] border-none
              flex items-center justify-center
              text-white
              transition-[background,transform,box-shadow] duration-200
              hover:bg-[#7352d4]
              active:scale-[0.94]
              ${this.recording ? 'recording' : ''}
            "
            @click=${this._onMicClick}>
            <svg class="w-[22px] h-[22px]" viewBox="0 0 24 24" fill="currentColor">
              <rect x="9" y="2" width="6" height="12" rx="3"/>
              <path d="M5 10v2a7 7 0 0 0 14 0v-2"
                    stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round"/>
              <path d="M12 19v3M9 22h6"
                    stroke="currentColor" stroke-width="1.6" fill="none" stroke-linecap="round"/>
            </svg>
          </button>

          <!-- Transcript toggle button -->
          <button
            title="${this.transcriptOpen ? this._t('controls.transcript_hide') : this._t('controls.transcript_show')}"
            aria-label="${this.transcriptOpen ? this._t('controls.transcript_panel_hide') : this._t('controls.transcript_panel_show')}"
            aria-pressed="${this.transcriptOpen}"
            @click=${this._onTranscriptToggle}
            class="
              w-12 h-12 rounded-full
              border-[0.5px]
              flex items-center justify-center
              transition-[background,color,border-color] duration-150
              active:scale-[0.94]
              ${this.transcriptOpen
        ? 'bg-[var(--bbl-accent2)] border-[var(--bbl-accent2)] text-white'
        : 'bg-[var(--bbl-btn-bg)] border-[var(--bbl-border)] text-[var(--bbl-text-muted)] hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]'}
            ">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.6" stroke-linecap="round"
                 stroke-linejoin="round" aria-hidden="true">
              <!-- outer frame -->
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <!-- vertical divider — right panel -->
              <line x1="15" y1="3" x2="15" y2="21"/>
            </svg>
          </button>

          <!-- End call button -->
          <button
            title="${this._t('controls.hangup')}"
            class="
              w-12 h-12 rounded-full
              bg-[var(--bbl-accent-dim)] border-[0.5px] border-[var(--bbl-accent-ring)]
              flex items-center justify-center
              text-[var(--bbl-accent)]
              transition-[background,transform] duration-150
              hover:bg-[rgba(224,68,106,0.25)]
              active:scale-[0.94]
            ">
            <svg class="w-[18px] h-[18px]" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="1.8">
              <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"
                    stroke-linecap="round"/>
              <line x1="4" y1="4" x2="20" y2="20" stroke-linecap="round"/>
            </svg>
          </button>

        </div>

      </footer>
    `;
  }
}
