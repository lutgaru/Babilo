/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { state } from 'lit/decorators.js';
import { AppView, SessionInfo, SessionSummary } from '../types/babilo';
import './bbl-config-list';
import './bbl-session';

export class BblShell extends LitElement {
  @state() private view: AppView = 'config-list';
  @state() private sessionInfo: SessionInfo | null = null;

  static styles = css`:host { display: block; }`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  private onSessionStarted(e: CustomEvent<SessionInfo>) {
    this.sessionInfo = e.detail;
    this.view = 'session';
  }

  private onSessionEnded(_e: CustomEvent<SessionSummary>) {
    this.sessionInfo = null;
    this.view = 'config-list';
  }

  render() {
    if (this.view === 'session' && this.sessionInfo) {
      return html`
        <bbl-session
          .sessionInfo=${this.sessionInfo}
          @session-ended=${this.onSessionEnded}>
        </bbl-session>
      `;
    }
    return html`
      <bbl-config-list
        @session-started=${this.onSessionStarted}>
      </bbl-config-list>
    `;
  }
}

customElements.define('bbl-shell', BblShell);