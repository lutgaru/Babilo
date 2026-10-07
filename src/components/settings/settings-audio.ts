import { LitElement, html } from 'lit';
import { property, state, customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader, numberInput, toggle, select } from './settings-truncate';
import type { HotkeySettings, TtsSettings } from '../../types/babilo';
import '../bbl-mic-panel';

export function prettyHotkey(code: string): string {
    if (code === 'Space') return 'Space';
    if (code.startsWith('Key')) return code.slice(3);
    if (code.startsWith('Digit')) return code.slice(5);
    if (code.startsWith('Numpad')) return 'Num ' + code.slice(6);
    return code
        .replace(/^Control(Left|Right)$/, 'Ctrl')
        .replace(/^Shift(Left|Right)$/, 'Shift')
        .replace(/^Alt(Left|Right)$/, 'Alt');
}

@customElement('bbl-settings-audio')
export class BblSettingsAudio extends withI18n(LitElement) {
    @property({ type: Object }) data: TtsSettings | null = null;
    @property({ type: Function }) onChange: (partial: Partial<TtsSettings>) => void = () => {};

    @property({ type: Object }) hotkey: HotkeySettings | null = null;
    @property({ type: Function }) onHotkeyChange: (partial: Partial<HotkeySettings>) => void = () => {};

    @state() private _capturing = false;

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        this._stopCapture();
    }

    private _captureHandler = (e: KeyboardEvent) => {
        e.preventDefault();
        e.stopPropagation();
        const code = e.code || e.key;
        this.onHotkeyChange({ key: code });
        this._stopCapture();
    };

    private _startCapture() {
        if (this._capturing) return;
        this._capturing = true;
        window.addEventListener('keydown', this._captureHandler, { capture: true });
    }

    private _stopCapture() {
        if (!this._capturing) return;
        this._capturing = false;
        window.removeEventListener('keydown', this._captureHandler, { capture: true });
    }

    private _renderHotkeySection() {
        const h = this.hotkey;
        if (!h) return html``;
        const disabled = !h.enabled;

        return html`
      ${sectionHeader(this._t('settings.hotkey.title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(
            this._t('settings.hotkey.enable'),
            this._t('settings.hotkey.enable_sub'),
            toggle(h.enabled, (v) => this.onHotkeyChange({ enabled: v }), 'hotkey-enable')
        )}
        ${row(
            this._t('settings.hotkey.mode'),
            this._t('settings.hotkey.mode_sub'),
            html`<div class="${disabled ? 'opacity-40 pointer-events-none' : ''}">
              ${select(h.mode, [
                  { value: 'toggle', label: this._t('settings.hotkey.mode_toggle') },
                  { value: 'push_to_talk', label: this._t('settings.hotkey.mode_ptt') },
              ], (v) => this.onHotkeyChange({ mode: v as HotkeySettings['mode'] }))}
            </div>`
        )}
        ${row(
            this._t('settings.hotkey.key'),
            h.mode === 'push_to_talk'
                ? this._t('settings.hotkey.key_sub_ptt')
                : this._t('settings.hotkey.key_sub_toggle'),
            html`
              <button
                ?disabled=${disabled}
                @click=${() => this._capturing ? this._stopCapture() : this._startCapture()}
                class="min-w-24 px-3 py-1.5 rounded-bbl-sm text-xs font-medium text-center
                       border outline-none cursor-pointer transition-colors duration-150
                       ${this._capturing
                           ? 'bg-bbl-accent2 border-bbl-accent2 text-white animate-pulse'
                           : 'bg-bbl-surface border-bbl-border text-bbl-text hover:border-bbl-accent2'}
                       disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-bbl-border">
                ${this._capturing ? this._t('settings.hotkey.press_key') : prettyHotkey(h.key)}
              </button>
            `
        )}
      </div>
    `;
    }

    render() {
        const s = this.data;
        return html`
      ${sectionHeader(this._t('settings.audio.input_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(
            this._t('settings.audio.microphone'),
            this._t('settings.audio.microphone_sub'),
            html`<bbl-mic-panel></bbl-mic-panel>`
        )}
      </div>

      ${this._renderHotkeySection()}

      ${!s ? html`
        ${sectionHeader(this._t('settings.tts.title'))}
        <div class="flex flex-col px-2 gap-0.5">
          <div class="px-5 py-4 text-sm text-bbl-text-faint italic">
            TTS engine not available
          </div>
        </div>
      ` : html`
        ${sectionHeader('AE ' + this._t('settings.tts.title'))}
        <div class="flex flex-col px-2 gap-0.5">
          ${row(
              this._t('settings.tts.ae_sample_rate'),
              this._t('settings.tts.ae_sample_rate_sub'),
              numberInput(s.ae.sample_rate, 8000, 48000, 1000, (v) => this.onChange({ ae: { ...s.ae, sample_rate: v } }), 'Hz')
          )}
          ${row(
              this._t('settings.tts.ae_chunk_size'),
              this._t('settings.tts.ae_chunk_size_sub'),
              numberInput(s.ae.base_chunk_size, 1, 256, 1, (v) => this.onChange({ ae: { ...s.ae, base_chunk_size: v } }))
          )}
        </div>

        ${sectionHeader('TTL ' + this._t('settings.tts.title'))}
        <div class="flex flex-col px-2 gap-0.5">
          ${row(
              this._t('settings.tts.ttl_compress'),
              this._t('settings.tts.ttl_compress_sub'),
              numberInput(s.ttl.chunk_compress_factor, 1, 32, 1, (v) => this.onChange({ ttl: { ...s.ttl, chunk_compress_factor: v } }))
          )}
          ${row(
              this._t('settings.tts.ttl_latent_dim'),
              this._t('settings.tts.ttl_latent_dim_sub'),
              numberInput(s.ttl.latent_dim, 1, 4096, 32, (v) => this.onChange({ ttl: { ...s.ttl, latent_dim: v } }))
          )}
        </div>
      `}
    `;
    }
}
