/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { startListening, stopAndProcess2, synthesize, testInference } from '../invoke';
import { listen } from '@tauri-apps/api/event';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-mic-panel';
import './bbl-controls';

export class BblShell extends LitElement {
  static properties = {
    aiState: { state: true },
    recording: { state: true },
    response: { state: true },
    timerSecs: { state: true },
    transcription: { state: true },
    corrections: { state: true },
    score: { state: true },
    nextStepHint: { state: true },
    messages: { state: true },
  };

  static styles = css`
    :host {
      display: grid;
      grid-template-rows: auto 1fr auto;
      height: 100dvh;
      margin: 0 auto;
      background: var(--bbl-bg);
      border-left: 0.5px solid var(--bbl-border);
      border-right: 0.5px solid var(--bbl-border);
    }
  `;

  constructor() {
    super();
    this.aiState = 'idle';
    this.recording = false;
    this.response = '';
    this.timerSecs = 0;
    this.transcription = '';
    this.corrections = [];
    this.score = null;
    this.nextStepHint = null;
    /** @type {Array<{ role: 'user' | 'ai'; content: string; timestamp?: number }>} */
    this.messages = [];
    /** @type {ReturnType<typeof setInterval> | null} */
    this._timerInterval = null;
    /** @type {Function | null} */
    this.unlistenStream = null;
  }

  /** @type {ReturnType<typeof setInterval> | null} */
  _timerInterval;
  /** @type {Function | null} */
  unlistenStream;

  startTimer() {
    this.timerSecs = 0;
    this._timerInterval = setInterval(() => { this.timerSecs++; }, 1000);
  }

  stopTimer() {
    if (this._timerInterval) clearInterval(this._timerInterval);
    this._timerInterval = null;
    this.timerSecs = 0;
  }

  /**
   * Add message to shared state (triggers re-render down the tree)
   * @private
   */
  _addMessage(role, content, timestamp) {
    this.messages = [...this.messages, { role, content, timestamp: timestamp ?? Date.now() }];
  }

  async speak(text) {
    this.response = 'Synthesizing...';
    try {
      const wavBytes = await synthesize(text);
      const uint8 = new Uint8Array(wavBytes.map((b) => b < 0 ? b + 256 : b));

      if (uint8.length < 44) throw new Error(`Datos muy cortos: ${uint8.length} bytes`);
      if (new TextDecoder().decode(uint8.slice(0, 4)) !== 'RIFF') throw new Error('Header WAV inválido');

      this.response = '▶ Speaking...';
      const audio = new Audio(URL.createObjectURL(new Blob([uint8], { type: 'audio/wav' })));

      audio.onloadeddata = () => console.log('✅ Audio cargado:', audio.duration, 's');
      audio.onerror = (e) => { this.response = `❌ Error: ${e.message}`; };
      audio.onended = () => { 
        this.response = '✓ Ready';
        // Add AI response to chat when TTS finishes
        this._addMessage('ai', text);
      };
      await audio.play();
    } catch (err) {
      console.error('❌ speak error:', err);
      this.response = `Error: ${err.message ?? JSON.stringify(err)}`;
    }
  }

  async toggleRecording() {
    const micPanel = this.shadowRoot?.querySelector('bbl-mic-panel');
    const selectedDevice = micPanel?.selectedDevice ?? null;

    if (!this.recording) {
      try {
        await startListening(selectedDevice);
        this.recording = true;
        this.aiState = 'listening';
        this.startTimer();
      } catch (err) {
        console.error('Error al iniciar micro:', err);
        this.aiState = 'idle';
        alert(`Error: ${err}`);
      }
    } else {
      const input = this.shadowRoot?.querySelector('#greet-input');
      const prompt = input?.value ?? '';
      this.aiState = 'thinking';

      this.unlistenStream = await listen('babilo://stream', ({ payload }) => {
        this.handleStreamEvent(payload);
      });

      try {
        await stopAndProcess2(prompt);
      } catch (err) {
        console.error('Error procesando audio:', err);
        this.aiState = 'idle';
        alert(`Error: ${err}`);
      } finally {
        this.recording = false;
        if (this.unlistenStream) this.unlistenStream();
        this.unlistenStream = null;
        this.stopTimer();
      }
    }
  }

  handleStreamEvent(event) {
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

        // ✅ Add user message to shared state
        this._addMessage('user', this.transcription);
        break;

      case 'error':
        console.error('Babilo stream error:', event.message);
        this.aiState = 'idle';
        alert(`Error: ${event.message}`);
        break;
    }
  }

  async handlePrompt(e) {
    const text = e.detail;
    try {
      this.aiState = 'processing';
      const res = await testInference(text);
      this.response = res;
      this.aiState = 'speaking';
      
      // Add user prompt to chat
      this._addMessage('user', text);
      await this.speak(res);
    } catch (err) {
      console.error('❌ testInference error:', err);
      this.response = `Error: ${err.message ?? JSON.stringify(err)}`;
      this.aiState = 'idle';
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    if (this.unlistenStream) this.unlistenStream();
    this.stopTimer();
  }

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

customElements.define('bbl-shell', BblShell);