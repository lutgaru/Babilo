/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { state } from 'lit/decorators.js';
import { startListening, stopAndProcess2, synthesize, testInference } from '../invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-mic-panel';
import './bbl-controls';

type AIState = 'idle' | 'listening' | 'thinking' | 'processing' | 'speaking';
type Message = { role: 'user' | 'ai'; content: string; timestamp?: number };
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
  @state() messages: Message[] = [];

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
  private _addMessage(role: 'user' | 'ai', content: string, timestamp?: number) {
    this.messages = [...this.messages, { role, content, timestamp: timestamp ?? Date.now() }];
  }

  // ── TTS / Audio ──
  private async speak(text: string) {
    this.response = 'Synthesizing...';
    try {
      const wavBytes = await synthesize(text);
      const uint8 = new Uint8Array(wavBytes.map((b) => (b < 0 ? b + 256 : b)));

      if (uint8.length < 44) throw new Error(`Datos muy cortos: ${uint8.length} bytes`);
      if (new TextDecoder().decode(uint8.slice(0, 4)) !== 'RIFF') throw new Error('Header WAV inválido');

      this.response = '▶ Speaking...';
      const audio = new Audio(URL.createObjectURL(new Blob([uint8], { type: 'audio/wav' })));

      audio.onloadeddata = () => console.log('✅ Audio cargado:', audio.duration, 's');
      audio.onerror = (e) => { this.response = `❌ Error: ${e.message}`; };
      audio.onended = () => { 
        this.response = '✓ Ready';
        this._addMessage('ai', text);
      };
      await audio.play();
    } catch (err: unknown) {
      console.error('❌ speak error:', err);
      const msg = err instanceof Error ? err.message : JSON.stringify(err);
      this.response = `Error: ${msg}`;
    }
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

        this._addMessage('user', this.transcription);
        break;

      case 'error':
        console.error('Babilo stream error:', event.message);
        this.aiState = 'idle';
        alert(`Error: ${event.message}`);
        break;
    }
  }

  // ── Prompt Handler ──
  private async handlePrompt(e: CustomEvent<string>) {
    const text = e.detail;
    try {
      this.aiState = 'processing';
      const res = await testInference(text);
      this.response = res;
      this.aiState = 'speaking';
      
      this._addMessage('user', text);
      await this.speak(res);
    } catch (err: unknown) {
      console.error('❌ testInference error:', err);
      const msg = err instanceof Error ? err.message : JSON.stringify(err);
      this.response = `Error: ${msg}`;
      this.aiState = 'idle';
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
      <bbl-top-bar
        .status=${statusText}
        .seconds=${this.timerSecs}
        ?active=${this.recording}>
      </bbl-top-bar>

      <bbl-stage 
        .aiState=${this.aiState} 
        .response=${this.response}
        .messages=${this.messages}>
      </bbl-stage>

      <bbl-controls
        ?recording=${this.recording}
        @mic-toggle=${this.toggleRecording}
        @prompt-submit=${this.handlePrompt}>
        <bbl-mic-panel slot="mic-panel"></bbl-mic-panel>
      </bbl-controls>
    `;
  }
}

// ── Stream Event Type (adjust to match your backend) ──
type StreamEvent =
  | { type: 'sentinel_reached' }
  | {
      type: 'analysis';
      data: {
        transcription: string;
        corrections: Correction[];
        score: number | null;
        next_step_hint?: string | null;
      };
    }
  | { type: 'error'; message: string };

customElements.define('bbl-shell', BblShell);