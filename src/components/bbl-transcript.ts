/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, query } from 'lit/decorators.js';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

export class BblTranscript extends LitElement {
  // ── Reactive Properties ──
  @property({ type: Array })
  messages: Array<{ role: 'user' | 'ai'; content: string; timestamp?: number }> = [];

  @property({ type: Number })
  maxMessages = 50;

  // ── DOM Element Query ──
  @query('.container')
  private _scrollContainer?: HTMLDivElement;

  static styles = css`
    :host {
      display: block;
      width: 100%;
      max-width: 440px;
    }

    /* ── Impossible in Tailwind ── */

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

  // Constructor removed: defaults are now handled by field initializers above
  // this.messages    = [];
  // this.maxMessages = 50;
  // this._scrollContainer = null;

  addMessage({ role, content, timestamp = Date.now() }: {
    role: 'user' | 'ai';
    content: string;
    timestamp?: number;
  }) {
    const newMessages = [...this.messages, { role, content, timestamp }];
    this.messages = newMessages.length > this.maxMessages
      ? newMessages.slice(-this.maxMessages)
      : newMessages;
    this.updateComplete.then(() => this._scrollToBottom());
  }

  clear() {
    this.messages = [];
  }

  private _formatTime(ts: number) {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  private _scrollToBottom() {
    if (this._scrollContainer) {
      this._scrollContainer.scrollTop = this._scrollContainer.scrollHeight;
    }
  }

  firstUpdated() {
    // @query automatically populates `_scrollContainer` after the first render
    this._scrollToBottom();
  }

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
            The conversation will appear here...
          </span>
        </div>
      `;
    }

    return html`
      <div class="container
                  flex flex-col gap-3
                  max-h-[200px] overflow-y-auto
                  p-1 scroll-smooth">
        ${this.messages.map((msg) => html`
          <div class="message
                      flex flex-col
                      ${msg.role === 'ai' ? 'items-start' : 'items-end'}">

            <div class="
              bg-[var(--bbl-btn-bg)] border-[0.5px] border-[var(--bbl-border)]
              py-[14px] px-[18px]
              text-[14px] leading-relaxed text-[var(--bbl-text)]
              max-w-[85%] break-words
              ${msg.role === 'ai'
                ? 'rounded-[var(--bbl-radius-md)] rounded-bl-[4px]'
                : 'rounded-[var(--bbl-radius-md)] rounded-br-[4px] bg-[var(--bbl-primary,var(--bbl-btn-bg))] border-[var(--bbl-primary-border,var(--bbl-border))] text-[var(--bbl-text-on-primary,var(--bbl-text))]'
              }
            ">
              ${msg.content}
            </div>

            ${msg.timestamp ? html`
              <span class="
                text-[10px] uppercase tracking-[0.1em]
                text-[var(--bbl-text-faint)]
                mt-1 mx-0.5
                ${msg.role === 'ai' ? 'self-start' : 'self-end'}
              ">
                ${this._formatTime(msg.timestamp)}
              </span>
            ` : ''}

          </div>
        `)}
      </div>
    `;
  }
}

customElements.define('bbl-transcript', BblTranscript);