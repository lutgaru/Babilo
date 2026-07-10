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
import './bbl-settings';

@customElement('bbl-shell')
export class BblShell extends withI18n(LitElement) {
  @state() private view: AppView = 'config-list';
  @state() private sessionInfo: SessionInfo | null = null;
  @state() private sessionPath = '';
  @state() private settingsOpen = false;

  // Core startup control states
  @state() private coreReady = false;
  @state() private coreError = '';
  @state() private splashMessage = 'Synchronizing Vulkan environments...';
  @state() private _selectedMic: string | null = null;

  private unlistenReady: UnlistenFn | null = null;
  private unlistenError: UnlistenFn | null = null;

  static styles = css`:host { display: block; }`;

  async connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);

    const isTauri = !!(window as any).__TAURI_INTERNALS__;

    if (isTauri) {
      try {
        this.unlistenReady = await listen('babilo://core-ready', () => {
          console.log('[Babilo Shell] Rust initialized successfully.');
          this.coreReady = true;
        });

        this.unlistenError = await listen<string>('babilo://core-error', (event) => {
          console.error('[Babilo Shell] Fatal error reported by backend:', event.payload);
          this.coreError = event.payload;
        });

        this.addEventListener('mic-change', (e: Event) => {
          console.log('[Babilo Shell] Mic changed:', (e as CustomEvent).detail.device);
          this._selectedMic = (e as CustomEvent<{ device: string | null }>).detail.device;
        });
      } catch (err) {
        this.coreError = `Error registering native listeners: ${err}`;
      }
    } else {
      console.log('[Babilo Shell Mock] Simulating Gemma weight loading (2s)...');

      setTimeout(() => {
        this.splashMessage = this._t('splash.load_vram');
      }, 100);

      setTimeout(() => {
        console.log('[Babilo Shell Mock] Emulation completed.');
        this.coreReady = true;
      }, 200);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this.unlistenReady) this.unlistenReady();
    if (this.unlistenError) this.unlistenError();
  }

  private onSessionStarted(e: CustomEvent<{ info: SessionInfo; path: string }>) {
    this.sessionInfo = e.detail.info;
    this.sessionPath = e.detail.path;
    this.view = 'session';
  }

  private onSessionEnded(_e: CustomEvent<SessionSummary>) {
    this.sessionInfo = null;
    this.view = 'config-list';
  }

  private onSettingsOpen() {
    this.settingsOpen = true;
  }

  private onSettingsClose() {
    this.settingsOpen = false;
  }

  render() {
    // 1. Splash mientras los motores no están listos
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

    // 2. bbl-settings se monta siempre encima de la vista activa
    //    y se anima con translateY según el atributo `open`.
    const settingsOverlay = html`
      <bbl-settings
        .open=${this.settingsOpen}
        @settings-close=${this.onSettingsClose}>
      </bbl-settings>
    `;

    if (this.view === 'session' && this.sessionInfo) {
      return html`
        <bbl-session
          .sessionInfo=${this.sessionInfo}
          .modePath=${this.sessionPath}
          .settingsOpen=${this.settingsOpen}
          .selectedMic=${this._selectedMic}
          @settings-open=${this.onSettingsOpen}
          @session-ended=${this.onSessionEnded}>
        </bbl-session>
        ${settingsOverlay}
      `;
    }

    return html`
      <bbl-config-list
        .settingsOpen=${this.settingsOpen}
        @session-started=${this.onSessionStarted}
        @settings-open=${this.onSettingsOpen}>
      </bbl-config-list>
      ${settingsOverlay}
    `;
  }
}