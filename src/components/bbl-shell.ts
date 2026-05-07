/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { state } from 'lit/decorators.js';
import { startListening, stopAndProcess2 } from '../invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { AIState, TranscriptMessage, StreamEvent, BabiloAnalysis } from '../types/babilo';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-mic-panel';
import './bbl-controls';

type Correction = unknown; // Adjust based on your actual correction structure

export class BblShell extends LitElement {
  // ── Reactive State Properties ──
  @state() aiState: AIState = 'idle';
  @state() recording = false;
  @state() response = '';
  @state() timerSecs = 0;
  @state() transcription = '';
  @state() corrections: Correction[] = [];
  @state() score: number | null = null;
  @state() nextStepHint: string | null = null;
  @state() messages: TranscriptMessage[] = [];

  // ── Private Fields (non-reactive) ──
  private _timerInterval: ReturnType<typeof setInterval> | null = null;
  private _unlistenStream: UnlistenFn | null = null;

  static styles = css`
    :host { display: block; }
    /* Solo CSS personalizado que NO puede ser utility */
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._unlistenStream) this._unlistenStream();
    this.stopTimer();
  }

  // ── Timer Helpers ──
  private startTimer() {
    this.timerSecs = 0;
    this._timerInterval = setInterval(() => { this.timerSecs++; }, 1000);
  }

  private stopTimer() {
    if (this._timerInterval) clearInterval(this._timerInterval);
    this._timerInterval = null;
    this.timerSecs = 0;
  }

  // ── Message Management ──
  private _addMessage(analysis: BabiloAnalysis) {
    this.messages = [...this.messages, { analysis, timestamp: Date.now() }];
  }

  // ── Recording Toggle ──
  async toggleRecording() {
    const micPanel = this.shadowRoot?.querySelector('bbl-mic-panel') as HTMLElement & { selectedDevice?: string | null };
    const selectedDevice = micPanel?.selectedDevice ?? null;

    if (!this.recording) {
      try {
        await startListening(selectedDevice);
        this.recording = true;
        this.aiState = 'listening';
        this.startTimer();
      } catch (err: unknown) {
        console.error('Error al iniciar micro:', err);
        this.aiState = 'idle';
        const msg = err instanceof Error ? err.message : String(err);
        alert(`Error: ${msg}`);
      }
    } else {
      const input = this.shadowRoot?.querySelector('#greet-input') as HTMLInputElement | null;
      const prompt = input?.value ?? '';
      this.aiState = 'thinking';

      this._unlistenStream = await listen('babilo://stream', ({ payload }) => {
        this.handleStreamEvent(payload as StreamEvent);
      });

      try {
        await stopAndProcess2(prompt);
      } catch (err: unknown) {
        console.error('Error procesando audio:', err);
        this.aiState = 'idle';
        const msg = err instanceof Error ? err.message : String(err);
        alert(`Error: ${msg}`);
      } finally {
        this.recording = false;
        this.aiState = 'idle';
        if (this._unlistenStream) this._unlistenStream();
        this._unlistenStream = null;
        this.stopTimer();
      }
    }
  }

  // ── Stream Event Handler ──
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

        console.log('📊 Analysis:', {
          transcription: this.transcription,
          corrections: this.corrections,
          score: this.score,
          nextStepHint: this.nextStepHint
        });

        this._addMessage(event.data);
        break;

      case 'error':
        console.error('Babilo stream error:', event.message);
        this.aiState = 'idle';
        alert(`Error: ${event.message}`);
        break;
    }
  }

  // ── Render ──
  render() {
    const statusText = this.recording
      ? 'Recording'
      : this.aiState === 'idle'
        ? 'Ready'
        : this.aiState;

    return html`
    <div class="flex flex-col min-h-screen bg-[var(--bbl-bg)] h-screen">
      <bbl-top-bar
        .status=${statusText}
        .seconds=${this.timerSecs}
        ?active=${this.recording}>
      </bbl-top-bar>
      <main class="flex-1 flex flex-col min-h-0 overflow-hidden">
        <bbl-stage 
          .aiState=${this.aiState} 
          .response=${this.response}
          .messages=${this.messages}
          class="h-full">
        </bbl-stage>
      </main>

      <bbl-controls 
        ?recording=${this.recording}
        @mic-toggle=${this.toggleRecording}
        class="flex-shrink-0">
        <bbl-mic-panel slot="mic-panel"></bbl-mic-panel>
      </bbl-controls>
    </div>
    `;
  }
}

customElements.define('bbl-shell', BblShell);