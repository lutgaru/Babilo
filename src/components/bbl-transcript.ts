/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';

export class BblTranscript extends LitElement {
  static properties = {
    messages: { type: Array },
    maxMessages: { type: Number },
  };

  static styles = css`
    :host {
      display: block;
      width: 100%;
      max-width: 440px;
      --bbl-chat-gap: 12px;
      --bbl-chat-padding: 14px 18px;
    }

    .container {
      display: flex;
      flex-direction: column;
      gap: var(--bbl-chat-gap);
      max-height: 200px;
      overflow-y: auto;
      padding: 4px;
      scroll-behavior: smooth;
    }

    .container::-webkit-scrollbar { width: 4px; }
    .container::-webkit-scrollbar-track { background: transparent; }
    .container::-webkit-scrollbar-thumb {
      background: var(--bbl-border);
      border-radius: 2px;
    }

    .message {
      display: flex;
      flex-direction: column;
      animation: fadeIn 0.2s ease-out;
    }

    @keyframes fadeIn {
      from { opacity: 0; transform: translateY(8px); }
      to { opacity: 1; transform: translateY(0); }
    }

    .bubble {
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      border-radius: var(--bbl-radius-md);
      padding: var(--bbl-chat-padding);
      font-size: 14px;
      line-height: 1.6;
      color: var(--bbl-text);
      max-width: 85%;
      word-wrap: break-word;
    }

    .message.ai { align-self: flex-start; }
    .message.ai .bubble { border-bottom-left-radius: 4px; }

    .message.user { align-self: flex-end; }
    .message.user .bubble {
      background: var(--bbl-primary, var(--bbl-btn-bg));
      border-color: var(--bbl-primary-border, var(--bbl-border));
      border-bottom-right-radius: 4px;
      color: var(--bbl-text-on-primary, var(--bbl-text));
    }

    .meta {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: var(--bbl-text-faint);
      margin: 4px 2px 0;
    }
    .message.ai .meta { align-self: flex-start; }
    .message.user .meta { align-self: flex-end; }

    .placeholder {
      color: var(--bbl-text-faint);
      font-style: italic;
    }

    .empty-state {
      text-align: center;
      padding: 24px 18px;
      color: var(--bbl-text-faint);
      font-size: 13px;
    }
  `;

  constructor() {
    super();
    /** @type {Array<{ role: 'user' | 'ai'; content: string; timestamp?: number }>} */
    this.messages = [];
    this.maxMessages = 50;
    /** @type {HTMLDivElement | null} */
    this._scrollContainer = null;
  }

  /** @type {HTMLDivElement | null} */
  _scrollContainer;

  /**
   * Add a new message to the transcript
   * @param {{ role: 'user' | 'ai'; content: string; timestamp?: number }} param0
   */
  addMessage({ role, content, timestamp = Date.now() }) {
    const newMessages = [...this.messages, { role, content, timestamp }];
    this.messages = newMessages.length > this.maxMessages
      ? newMessages.slice(-this.maxMessages)
      : newMessages;
    this.updateComplete.then(() => this._scrollToBottom());
  }

  /** Clear all messages */
  clear() {
    this.messages = [];
  }

  /**
   * Format timestamp to readable time
   * @private
   */
  _formatTime(ts) {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  /**
   * Scroll to bottom of container
   * @private
   */
  _scrollToBottom() {
    if (this._scrollContainer) {
      this._scrollContainer.scrollTop = this._scrollContainer.scrollHeight;
    }
  }

  firstUpdated() {
    this._scrollContainer = this.shadowRoot?.querySelector('.container');
    this._scrollToBottom();
  }

  updated(changedProps) {
    if (changedProps.has('messages')) {
      this._scrollToBottom();
    }
  }

  render() {
    if (this.messages.length === 0) {
      return html`
        <div class="box">
          <div class="empty-state">
            <span class="placeholder">The conversation will appear here...</span>
          </div>
        </div>
      `;
    }

    return html`
      <div class="container">
        ${this.messages.map((msg) => html`
          <div class="message ${msg.role}">
            <div class="bubble">${msg.content}</div>
            ${msg.timestamp 
              ? html`<span class="meta">${this._formatTime(msg.timestamp)}</span>` 
              : ''}
          </div>
        `)}
      </div>
    `;
  }
}

customElements.define('bbl-transcript', BblTranscript);