/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { state, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { AppView, SessionInfo, SessionSummary } from '../types/babilo';
import './bbl-config-list';
import './bbl-session';
import './bbl-splash';

@customElement('bbl-shell')
export class BblShell extends withI18n(LitElement) {
  @state() private view: AppView = 'config-list';
  @state() private sessionInfo: SessionInfo | null = null;

  // Core startup control states
  @state() private coreReady = false;
  @state() private coreError = '';
  @state() private splashMessage = 'Synchronizing Vulkan environments...';

  private unlistenReady: UnlistenFn | null = null;
  private unlistenError: UnlistenFn | null = null;

  static styles = css`:host { display: block; }`;

  async connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);

    const isTauri = !!(window as any).__TAURI_INTERNALS__;

    if (isTauri) {
      // REAL WORLD (TAURI + RUST BACKGROUND THREAD)
      try {
        this.unlistenReady = await listen('babilo://core-ready', () => {
          console.log('[Babilo Shell] Rust initialized successfully.');
          this.coreReady = true;
        });

        this.unlistenError = await listen<string>('babilo://core-error', (event) => {
          console.error('[Babilo Shell] Fatal error reported by backend:', event.payload);
          this.coreError = event.payload;
        });
      } catch (err) {
        this.coreError = `Error registering native listeners: ${err}`;
      }
    } else {
      // MOCK WORLD (Pure Browser for fast Dev)
      console.log('[Babilo Shell Mock] Simulating Gemma weight loading (2s)...');

      setTimeout(() => {
        this.splashMessage = this._t('splash.load_vram');
      }, 1000);

      setTimeout(() => {
        console.log('[Babilo Shell Mock] Emulation completed.');
        this.coreReady = true;
      }, 2200);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this.unlistenReady) this.unlistenReady();
    if (this.unlistenError) this.unlistenError();
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
    // 1. If heavy engines are not ready or failed, force Splashscreen
    if (!this.coreReady) {
      return html`
        <bbl-splash 
          .message=${this.splashMessage === 'Synchronizing Vulkan environments...'
          ? this._t('splash.sync_vulkan')
          : this.splashMessage}
          .errorMessage=${this.coreError}>
        </bbl-splash>
      `;
    }

    // 2. Normal screen flow once app is loaded
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