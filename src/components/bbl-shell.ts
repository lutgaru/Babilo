/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { startListening, stopAndProcess, synthesize, testInference } from '../invoke';
import type { AIState } from './bbl-stage';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-mic-panel';
import './bbl-controls';

export class BblShell extends LitElement {
  static properties = {
    aiState:   { state: true },
    recording: { state: true },
    response:  { state: true },
    timerSecs: { state: true },
  };

  constructor() {
    super();
    this.aiState   = 'idle' as AIState;
    this.recording = false;
    this.response  = '';
    this.timerSecs = 0;
  }

  static styles = css`
    :host {
      display: grid;
      grid-template-rows: auto 1fr auto;
      height: 100dvh;
      margin: 0 auto;
      background: var(--bbl-bg);
      border-left:  0.5px solid var(--bbl-border);
      border-right: 0.5px solid var(--bbl-border);
    }
  `;

  private _timerInterval: ReturnType<typeof setInterval> | null = null;

  private startTimer() {
    this.timerSecs = 0;
    this._timerInterval = setInterval(() => { this.timerSecs++; }, 1000);
  }

  private stopTimer() {
    if (this._timerInterval) clearInterval(this._timerInterval);
    this.timerSecs = 0;
  }

  private async speak(text: string) {
    this.response = 'Sintetizando...';
    try {
      const wavBytes = await synthesize(text);
      const uint8    = new Uint8Array(wavBytes.map((b: number) => b < 0 ? b + 256 : b));

      if (uint8.length < 44) throw new Error(`Datos muy cortos: ${uint8.length} bytes`);
      if (new TextDecoder().decode(uint8.slice(0, 4)) !== 'RIFF') throw new Error('Header WAV inválido');

      this.response = '▶ Reproduciendo...';
      const audio   = new Audio(URL.createObjectURL(new Blob([uint8], { type: 'audio/wav' })));

      audio.onloadeddata = () => console.log('✅ Audio cargado:', audio.duration, 's');
      audio.onerror      = (e: any) => { this.response = `❌ Error: ${e.message}`; };
      audio.onended      = ()       => { this.response = '✓ Listo'; };
      await audio.play();
    } catch (err: any) {
      console.error('❌ speak error:', err);
      this.response = `Error: ${err.message ?? JSON.stringify(err)}`;
    }
  }

  private async toggleRecording() {
    const micPanel       = this.shadowRoot!.querySelector('bbl-mic-panel') as any;
    const selectedDevice = micPanel?.selectedDevice ?? null;

    if (!this.recording) {
      try {
        await startListening(selectedDevice);
        this.recording = true;
        this.aiState   = 'listening';
        this.startTimer();
      } catch (err) {
        console.error('Error al iniciar micro:', err);
        alert(`Error: ${err}`);
      }
    } else {
      const prompt = (this.shadowRoot!.querySelector('#greet-input') as HTMLInputElement)?.value ?? '';
      try {
        const response = await stopAndProcess(prompt);
        this.aiState  = 'speaking';
        this.response = response;
        await this.speak(response);
      } catch (err) {
        console.error('Error procesando audio:', err);
      } finally {
        this.recording = false;
        this.aiState   = 'idle';
        this.stopTimer();
      }
    }
  }

  private async handlePrompt(e: CustomEvent) {
    const text = e.detail as string;
    try {
      this.aiState  = 'processing';
      const res     = await testInference(text);
      this.response = res;
      this.aiState  = 'speaking';
      await this.speak(res);
    } catch (err: any) {
      console.error('❌ testInference error:', err);
      this.response = `Error: ${err.message ?? JSON.stringify(err)}`;
    } finally {
      this.aiState = 'idle';
    }
  }

  render() {
    return html`
      <bbl-top-bar
        .status=${this.recording ? 'grabando' : this.aiState === 'idle' ? 'listo' : this.aiState}
        .seconds=${this.timerSecs}
        ?active=${this.recording}>
      </bbl-top-bar>

      <bbl-stage .aiState=${this.aiState} .response=${this.response}></bbl-stage>

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