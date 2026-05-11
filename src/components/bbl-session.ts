/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { property, state } from 'lit/decorators.js';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  AIState, BabiloAnalysis, SessionInfo,
  SessionSummary, StreamEvent, TranscriptMessage
} from '../types/babilo';
import { endSession, startListening, stopAndProcess2 } from '../invoke';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-controls';
import './bbl-mic-panel';

export class BblSession extends LitElement {
  // ── Props desde el router ──
  @property({ type: Object }) sessionInfo!: SessionInfo;

  // ── Estado reactivo ──
  @state() aiState: AIState = 'idle';
  @state() recording = false;
  @state() timerSecs = 0;
  @state() transcription = '';
  @state() corrections: BabiloAnalysis['corrections'] = [];
  @state() score: number | null = null;
  @state() nextStepHint: string | null = null;
  @state() messages: TranscriptMessage[] = [];
  @state() response = '';

  private _timerInterval: ReturnType<typeof setInterval> | null = null;
  private _unlistenStream: UnlistenFn | null = null;

  static styles = css`:host { display: block; }`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);

    // If the mode speaks first, show the opening line immediately
    if (this.sessionInfo.opening_line) {
      this.response = this.sessionInfo.opening_line;
      this.aiState = 'speaking';
      // After showing it, return to idle so the user can respond
      setTimeout(() => { this.aiState = 'idle'; }, 1200);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._unlistenStream?.();
    this.stopTimer();
  }

  // ── Timer ──
  private startTimer() {
    this.timerSecs = 0;
    this._timerInterval = setInterval(() => { this.timerSecs++; }, 1000);
  }

  private stopTimer() {
    if (this._timerInterval) clearInterval(this._timerInterval);
    this._timerInterval = null;
    this.timerSecs = 0;
  }

  // ── Colgar ──
  private async hangUp() {
    this._unlistenStream?.();
    this.stopTimer();
    try {
      const summary: SessionSummary = await endSession();
      this.dispatchEvent(new CustomEvent<SessionSummary>('session-ended', {
        detail: summary,
        bubbles: true,
        composed: true,
      }));
    } catch (e) {
      console.error('Error ending session:', e);
      // Return to the list anyway
      this.dispatchEvent(new CustomEvent('session-ended', {
        detail: { session_id: '', mode_name: '', turns: 0, average_score: 0 },
        bubbles: true,
        composed: true,
      }));
    }
  }

  // ── Recording (audio) ──
  async toggleRecording() {
    const micPanel = this.shadowRoot?.querySelector('bbl-mic-panel') as
      HTMLElement & { selectedDevice?: string | null };
    const selectedDevice = micPanel?.selectedDevice ?? null;

    if (!this.recording) {
      try {
        await startListening(selectedDevice);
        this.recording = true;
        this.aiState = 'listening';
        this.startTimer();
      } catch (err) {
        console.error('Error starting mic:', err);
        this.aiState = 'idle';
      }
    } else {
      this.aiState = 'thinking';
      this._unlistenStream = await listen('babilo://stream', ({ payload }) => {
        this.handleStreamEvent(payload as StreamEvent);
      });
      try {
   // The system prompt already comes from the active mode in the backend —
    // this prompt is just for compatibility with the existing signature
        await stopAndProcess2('');
      } catch (err) {
        console.error('Error procesando audio:', err);
        this.aiState = 'idle';
      } finally {
        this.recording = false;
        this._unlistenStream?.();
        this._unlistenStream = null;
        this.stopTimer();
      }
    }
  }

  // ── Text input (for accepts_text modes) ──
  private async submitText(text: string) {
    if (!text.trim()) return;
    this.aiState = 'thinking';
    this._unlistenStream = await listen('babilo://stream', ({ payload }) => {
      this.handleStreamEvent(payload as StreamEvent);
    });
    try {
      await stopAndProcess2(text);
    } catch (err) {
      console.error('Error procesando texto:', err);
      this.aiState = 'idle';
    } finally {
      this._unlistenStream?.();
      this._unlistenStream = null;
    }
  }

  // ── Stream events ──
  private handleStreamEvent(event: StreamEvent) {
    switch (event.type) {
      case 'sentinel_reached':
        this.aiState = 'speaking';
        break;

      case 'analysis':
        this.transcription = event.data.transcription;
        this.corrections = event.data.corrections;
        this.score = event.data.score;
        this.nextStepHint = event.data.next_step_hint ?? null;
        this.messages = [...this.messages, {
          analysis: event.data,
          timestamp: Date.now(),
        }];
        this.aiState = 'idle';
        break;

      case 'error':
        console.error('Babilo stream error:', event.message);
        this.aiState = 'idle';
        break;
    }
  }

  // ── Input zone based on caps ──
  private renderInputZone() {
    const { accepts_audio, accepts_text } = this.sessionInfo.caps;

    // Text mode
    if (accepts_text && !accepts_audio) {
      return html`
        <div class="flex gap-2 px-4 py-3 border-t border-[var(--bbl-border)]">
          <input
            id="text-input"
            type="text"
            placeholder="Type your answer..."
            ?disabled=${this.aiState === 'thinking' || this.aiState === 'speaking'}
            @keydown=${(e: KeyboardEvent) => {
              if (e.key === 'Enter') {
                const input = e.target as HTMLInputElement;
                this.submitText(input.value);
                input.value = '';
              }
            }}
            class="flex-1 bg-[var(--bbl-surface)] border border-[var(--bbl-border)]
                   rounded-xl px-4 py-2 text-sm text-[var(--bbl-text)]
                   placeholder:text-[var(--bbl-muted)] outline-none
                   focus:border-[var(--bbl-accent)] transition-colors"/>
          <button
            @click=${() => {
              const input = this.shadowRoot?.querySelector('#text-input') as HTMLInputElement;
              if (input) { this.submitText(input.value); input.value = ''; }
            }}
            ?disabled=${this.aiState === 'thinking' || this.aiState === 'speaking'}
            class="px-4 py-2 rounded-xl bg-[var(--bbl-accent)] text-white text-sm
                   disabled:opacity-40 transition-opacity">
            Send
          </button>
        </div>
      `;
    }

    // Audio mode (default) — reuses existing bbl-controls
    return html`
      <bbl-controls
        ?recording=${this.recording}
        @mic-toggle=${this.toggleRecording}
        class="flex-shrink-0">
        <bbl-mic-panel slot="mic-panel"></bbl-mic-panel>
      </bbl-controls>
    `;
  }

  render() {
    const statusText = this.recording ? 'Recording'
      : this.aiState === 'idle' ? 'Ready'
      : this.aiState;

    return html`
      <div class="flex flex-col h-screen bg-[var(--bbl-bg)]">

        <!-- Top bar con nombre del modo y botón colgar -->
        <bbl-top-bar
          .status=${statusText}
          .seconds=${this.timerSecs}
          .modeName=${this.sessionInfo.mode_name}
          ?active=${this.recording}
          @hang-up=${this.hangUp}>
        </bbl-top-bar>

        <!-- Main content -->
        <main class="flex-1 flex flex-col min-h-0 overflow-hidden">
          <bbl-stage
            .aiState=${this.aiState}
            .response=${this.response}
            .messages=${this.messages}
            class="h-full">
          </bbl-stage>
        </main>

        <!-- Input zone — changes based on mode caps -->
        ${this.renderInputZone()}

      </div>
    `;
  }
}

customElements.define('bbl-session', BblSession);