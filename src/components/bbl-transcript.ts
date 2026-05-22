/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, query, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { BabiloAnalysis, Correction, TranscriptMessage } from '../types/babilo';

@customElement('bbl-transcript')
export class BblTranscript extends withI18n(LitElement) {
  @property({ type: Array })
  messages: TranscriptMessage[] = [];

  @property({ type: Number })
  maxMessages = 50;

  @query('.container')
  private _scrollContainer?: HTMLDivElement;

  static styles = css`
  :host {
    display: block;
    width: 100%;
  }

  /* ── Only what Tailwind CAN'T do in shadow DOM ── */
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0);   }
  }
  .message { animation: fadeIn 0.2s ease-out; }

  .container::-webkit-scrollbar       { width: 4px; }
  .container::-webkit-scrollbar-track { background: transparent; }
  .container::-webkit-scrollbar-thumb {
    background: var(--bbl-border);
    border-radius: 2px;
  }
`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  addMessage(analysis: BabiloAnalysis) {
    const newMessages: TranscriptMessage[] = [
      ...this.messages,
      { analysis, timestamp: Date.now() }
    ];
    this.messages = newMessages.length > this.maxMessages
      ? newMessages.slice(-this.maxMessages)
      : newMessages;
    this.updateComplete.then(() => this._scrollToBottom());
  }

  clear() { this.messages = []; }

  private _formatTime(ts: number) {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  private _scrollToBottom() {
    if (this._scrollContainer) {
      this._scrollContainer.scrollTop = this._scrollContainer.scrollHeight;
    }
  }

  private _getScoreClasses(score: number): string {
    if (score >= 8) return 'bg-[#22c55e] text-white';
    if (score >= 5) return 'bg-[#f59e0b] text-white';
    return 'bg-[var(--bbl-accent)] text-white';
  }

  private _renderCorrection(correction: Correction) {
    return html`
      <div class="
        flex items-center gap-1.5
        px-2 py-1
        bg-[var(--bbl-btn-bg)]
        rounded-[var(--bbl-radius-sm)]
        text-[12px]
      ">
        <span class="
          line-through
          text-[var(--bbl-accent)]
          opacity-80
        ">
          ${correction.original}
        </span>
        <span class="
          text-[10px]
          text-[var(--bbl-text-muted)]
        ">→</span>
        <span class="
          font-medium
          text-[var(--bbl-text)]
        ">
          ${correction.fixed}
        </span>
        ${correction.reason ? html`
          <span class="
            text-[11px]
            text-[var(--bbl-text-faint)]
            italic
            ml-auto
          ">
            (${correction.reason})
          </span>
        ` : ''}
      </div>
    `;
  }

  firstUpdated() { this._scrollToBottom(); }

  updated(changedProps: Map<string, unknown>) {
    if (changedProps.has('messages')) {
      this._scrollToBottom();
    }
  }

  render() {
    if (this.messages.length === 0) {
      return html`
        <div class="flex items-center justify-center py-6 px-[18px]">
          <span class="text-[13px] italic text-[var(--bbl-text-faint)]">
            ${this._t('transcript.empty')}
          </span>
        </div>
      `;
    }

    return html`
         <div class="container
                flex flex-col gap-4
                h-full min-h-0
                overflow-y-auto
                p-1 scroll-smooth
                pr-2"> 
        ${this.messages.map((msg) => html`
          <div class="message flex flex-col gap-2">
            
            <!-- 🗣️ User transcription (right) -->
            <div class="flex flex-col items-end">
              <div class="
                bg-[var(--bbl-btn-bg)] border-[0.5px] border-[var(--bbl-border)]
                py-[14px] px-[18px]
                text-[14px] leading-relaxed text-[var(--bbl-text)]
                max-w-[85%] break-words
                rounded-[var(--bbl-radius-md)] rounded-br-[4px]
              ">
                ${msg.analysis.transcription}
              </div>
              <span class="text-[10px] uppercase tracking-[0.1em] text-[var(--bbl-text-faint)] mt-1">
                ${this._formatTime(msg.timestamp)}
              </span>
            </div>

            <div class="
              flex flex-col gap-2
              bg-[var(--bbl-surface-alt)] border-[0.5px] border-[var(--bbl-border)]
              rounded-[var(--bbl-radius-md)] rounded-bl-[4px]
              p-3 max-w-[90%]
            ">
              <!-- Score badge -->
              <div class="flex items-center gap-2">
                <span class="
                  inline-flex items-center gap-1
                  px-2 py-0.5
                  rounded-full
                  text-[12px] font-semibold
                  ${this._getScoreClasses(msg.analysis.score)}
                ">
                  ${msg.analysis.score}/10
                </span>
                <span class="text-[11px] uppercase tracking-[0.1em] text-[var(--bbl-text-faint)]">
                  ${this._t('transcript.analysis')}
                </span>
              </div>

              <!-- Corrections list -->
              ${msg.analysis.corrections.length > 0 ? html`
                <div class="flex flex-col gap-1">
                  ${msg.analysis.corrections.map((c) => this._renderCorrection(c))}
                </div>
              ` : html`
                <span class="text-[12px] text-[var(--bbl-text-muted)] italic">
                  ${this._t('transcript.no_corrections')}
                </span>
              `}

              <!-- Next step hint -->
              ${msg.analysis.next_step_hint ? html`
                <div class="
                  bg-[var(--bbl-surface)] 
                  border-l-2 border-l-[var(--bbl-accent2)]
                  pl-3 pr-2 py-2
                  rounded-r-[var(--bbl-radius-sm)]
                  text-[13px] text-[var(--bbl-text-muted)]
                ">
                  💡 ${msg.analysis.next_step_hint}
                </div>
              ` : ''}

              <!-- AI response -->
              ${msg.analysis.response ? html`
                <div class="
                  mt-1 pt-2 border-t border-[var(--bbl-border)]
                  text-[14px] leading-relaxed text-[var(--bbl-text)]
                ">
                  ${msg.analysis.response}
                </div>
              ` : ''}
            </div>

          </div>
        `)}
      </div>
    `;
  }
}