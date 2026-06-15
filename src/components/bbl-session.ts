/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import { LitElement, html, css } from 'lit';
import { property, state, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  AIState, AiStateEvent, BabiloAnalysis, SessionInfo,
  SessionSummary, StreamEvent, TranscriptMessage
} from '../types/babilo';
import { endSession, startListening, stopAndProcessStreaming, processTextStreaming, startSession } from '../invoke';
import './bbl-top-bar';
import './bbl-stage';
import './bbl-controls';
import './bbl-mic-panel';
import './bbl-transcript';


@customElement('bbl-session')
export class BblSession extends withI18n(LitElement) {
  // ── Props desde el router ──
  @property({ type: Object }) sessionInfo!: SessionInfo;

  /** Passed down from bbl-shell to keep the gear icon active */
  @property({ type: Boolean }) settingsOpen = false;

  /** Mode file path used to restart the session on reset */
  @property({ type: String }) modePath = '';

  @property({ attribute: false }) selectedMic: string | null = null;

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

  /** Controls the side-panel visibility; open by default */
  @state() transcriptOpen = true;

  private _timerInterval: ReturnType<typeof setInterval> | null = null;
  private _unlistenStream: UnlistenFn | null = null;
  private _unlistenAiState: UnlistenFn | null = null;

  static styles = css`:host { display: block; }`;

  // Bind handlers once so Lit can add/remove the same reference
  constructor() {
    super();
    this.hangUp = this.hangUp.bind(this);
    this.toggleRecording = this.toggleRecording.bind(this);
    this._onSettingsOpen = this._onSettingsOpen.bind(this);
    this._onResetContext = this._onResetContext.bind(this);
  }

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);

    // Listen for backend-driven state changes (source of truth for AI state)
    this._setupAiStateListener();

    // If the mode speaks first, show the opening line immediately
    if (this.sessionInfo.caps.llm_initiates) {
      this.aiState = 'thinking';
      console.log('Opening line:', this.sessionInfo.opening_line);
      // Register stream listener to capture user response events
      this._setupStreamListener();

      // Backend will drive the full cycle: thinking → speaking → idle
      // via babilo://ai-state events emitted from run_turn_streaming
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._cleanupStreamListener();
    this._cleanupAiStateListener();
    this.stopTimer();
  }

  // ── Stream listener management (DRY) ──
  private _setupStreamListener() {
    this._cleanupStreamListener();

    const isTauri = !!(window as any).__TAURI_INTERNALS__;
    const processEvent = (event: StreamEvent) => this.handleStreamEvent(event);

    if (isTauri) {
      listen('babilo://stream', ({ payload }) => {
        processEvent(payload as StreamEvent);
      }).then(fn => {
        this._unlistenStream = fn;
      }).catch(err => {
        console.error('Error setting up native stream listener:', err);
      });
    } else {
      const mockHandler = (e: Event) => {
        const detail = (e as CustomEvent).detail;
        if (detail?.payload) processEvent(detail.payload as StreamEvent);
      };
      window.addEventListener('babilo://stream', mockHandler);
      this._unlistenStream = () => window.removeEventListener('babilo://stream', mockHandler);
    }
  }

  private _cleanupStreamListener() {
    if (this._unlistenStream) {
      this._unlistenStream();
      this._unlistenStream = null;
    }
  }

  /**
   * Listen for backend-driven AI state changes.
   * Backend is the source of truth — these events override any optimistic
   * frontend state to keep everything in sync (especially after TTS finishes).
   */
  private _setupAiStateListener() {
    const isTauri = !!(window as any).__TAURI_INTERNALS__;

    const onState = (state: AiStateEvent) => {
      this.aiState = state;
    };

    if (isTauri) {
      listen('babilo://ai-state', ({ payload }) => {
        onState(payload as AiStateEvent);
      }).then(fn => {
        this._unlistenAiState = fn;
      }).catch(err => {
        console.error('Error setting up ai-state listener:', err);
      });
    } else {
      const mockHandler = (e: Event) => {
        const detail = (e as CustomEvent).detail;
        if (detail?.payload) onState(detail.payload as AiStateEvent);
      };
      window.addEventListener('babilo://ai-state', mockHandler);
      this._unlistenAiState = () => window.removeEventListener('babilo://ai-state', mockHandler);
    }
  }

  private _cleanupAiStateListener() {
    if (this._unlistenAiState) {
      this._unlistenAiState();
      this._unlistenAiState = null;
    }
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

  // ── Hang up — called from bbl-controls @hang-up ──
  async hangUp() {
    this._cleanupStreamListener();
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

  // ── Reset context — called from bbl-controls @reset-context ──
  async _onResetContext() {
    try {
      await endSession();
      await startSession(this.modePath);
      this.messages = [];
      this.response = '';
      this.score = null;
      this.corrections = [];
      this.nextStepHint = null;
    } catch (e) {
      console.error('Error resetting context:', e);
    }
  }

  // ── Settings — re-emit upward to bbl-shell ──
  private _onSettingsOpen() {
    this.dispatchEvent(new CustomEvent('settings-open', {
      bubbles: true,
      composed: true,
    }));
  }

  // ── Recording ──
  async toggleRecording() {
    const selectedDevice = this.selectedMic;

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
      this.recording = false;
      this.stopTimer();

      // Setup stream listener for backend response
      this._setupStreamListener();

      try {
        await stopAndProcessStreaming('');
      } catch (err) {
        console.error('Error procesando audio:', err);
        this.aiState = 'idle';
      }
    }
  }

  // ── Text input (for accepts_text modes) ──
  private async submitText(text: string) {
    if (!text.trim()) return;
    this.aiState = 'thinking';

    // Setup stream listener for backend response
    this._setupStreamListener();

    try {
      await processTextStreaming(text);
    } catch (err) {
      console.error('Error procesando texto:', err);
      this.aiState = 'idle';
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
        // AI state is now driven by backend via babilo://ai-state events
        break;

      case 'error':
        console.error('Babilo stream error:', event.message);
        this.aiState = 'idle';
        break;
    }
  }

  // ── Side panel ──
  private _renderSidePanel() {
    return html`
      <div
        aria-hidden="${!this.transcriptOpen}"
        class="
          flex-shrink-0 flex flex-col min-h-0 overflow-hidden
          border-l-[0.5px] border-[var(--bbl-border)]
          bg-[var(--bbl-surface)]
          transition-[width,opacity] duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]
          ${this.transcriptOpen
        ? 'w-[min(420px,40vw)] opacity-100'
        : 'w-0 opacity-0 pointer-events-none border-l-0'}
        ">

        <!-- Header -->
        <div class="
          flex items-center justify-between
          px-4 py-3
          border-b-[0.5px] border-[var(--bbl-border)]
          flex-shrink-0">
          <span class="
            text-[11px] font-semibold tracking-[0.08em] uppercase
                       text-[var(--bbl-text-faint)]">
            Transcript
          </span>
          <button
            title="${this._t('common.close')}"
            aria-label="${this._t('common.close')}"
            @click=${() => { this.transcriptOpen = false; }}
            class="
              flex items-center justify-center w-7 h-7 rounded-md
                   text-[var(--bbl-text-muted)]
                   hover:bg-[var(--bbl-btn-hover)] hover:text-[var(--bbl-text)]
              transition-[background,color] duration-150
              cursor-pointer">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round"
                 aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="flex-1 min-h-0 overflow-hidden flex flex-col">
          <bbl-transcript
            .messages=${this.messages}
            class="w-full h-full">
          </bbl-transcript>
        </div>

      </div>
    `;
  }

  // ── Input zone based on caps ──
  private renderInputZone() {
    const { accepts_audio, accepts_text } = this.sessionInfo.caps;

    // Text mode
    if (accepts_text && !accepts_audio) {
      return html`
        <bbl-controls
          ?textMode=${true}
          .transcriptOpen=${this.transcriptOpen}
          @transcript-toggle=${() => { this.transcriptOpen = !this.transcriptOpen; }}
          @hang-up=${this.hangUp}
          @reset-context=${this._onResetContext}
          class="flex-shrink-0">
        </bbl-controls>
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
            ${this._t('common.send')}
          </button>
        </div>
      `;
    }

    // Audio mode (default) — reuses existing bbl-controls
    return html`
      <bbl-controls
        ?recording=${this.recording}
        .transcriptOpen=${this.transcriptOpen}
        @mic-toggle=${this.toggleRecording}
        @transcript-toggle=${() => { this.transcriptOpen = !this.transcriptOpen; }}
        @hang-up=${this.hangUp}
        @reset-context=${this._onResetContext}
        class="flex-shrink-0">
      </bbl-controls>
    `;
  }

  render() {
    const statusText = this.recording ? 'Recording'
      : this.aiState === 'idle' ? this._t('state.idle')
        : this._t(`state.${this.aiState}` as any);

    return html`
      <div class="flex flex-col h-screen bg-[var(--bbl-bg)]">

        <!-- Top bar con nombre del modo y botón colgar -->
        <bbl-top-bar
          .status=${statusText}
          .seconds=${this.timerSecs}
          .modeName=${this.sessionInfo.mode_name}
          .settingsOpen=${this.settingsOpen}
          ?active=${this.recording}
          @settings-open=${this._onSettingsOpen}>
        </bbl-top-bar>

        <!-- session-body: flex-row so bbl-stage and the side panel sit side by side -->
        <div class="flex-1 flex flex-row min-h-0 overflow-hidden">

          <main class="flex-1 flex flex-col min-h-0 overflow-hidden">
            <bbl-stage
              .aiState=${this.aiState}
              .response=${this.response}
              class="h-full">
            </bbl-stage>
          </main>

          <!-- Side panel pushes bbl-stage left as it opens -->
          ${this._renderSidePanel()}

        </div>

        ${this.renderInputZone()}

      </div>
    `;
  }
}
