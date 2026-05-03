/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';

export class BblTranscript extends LitElement {
  static properties = {
    text: {},
  };

  constructor() {
    super();
    this.text = '';
  }

  static styles = css`
    :host {
      display: block;
      width: 100%;
      max-width: 440px;
    }

    .box {
      background: var(--bbl-btn-bg);
      border: 0.5px solid var(--bbl-border);
      border-radius: var(--bbl-radius-md);
      padding: 14px 18px;
      min-height: 72px;
    }

    .label {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.1em;
      color: var(--bbl-text-faint);
      margin-bottom: 6px;
    }

    p {
      font-size: 14px;
      line-height: 1.6;
      color: var(--bbl-text);
    }

    .placeholder {
      color: var(--bbl-text-faint);
      font-style: italic;
    }
  `;

  render() {
    return html`
      <div class="box">
        <div class="label">Response</div>
        <p id="greet-msg">
          ${this.text
            ? this.text
            : html`<span class="placeholder">The AI response will appear here...</span>`}
        </p>
      </div>
    `;
  }
}

customElements.define('bbl-transcript', BblTranscript);