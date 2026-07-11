import { LitElement, html } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader, numberInput } from './settings-truncate';
import '../bbl-mic-panel';
import type { AudioSettings } from '../../types/babilo';

@customElement('bbl-settings-audio')
export class BblSettingsAudio extends withI18n(LitElement) {
    @property({ type: Object }) data: AudioSettings | null = null;
    @property({ type: Function }) onChange: (partial: Partial<AudioSettings>) => void = () => {};

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    render() {
        if (!this.data) return html``;
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

      ${sectionHeader(this._t('settings.audio.advanced_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(
            this._t('settings.audio.sample_rate'),
            this._t('settings.audio.sample_rate_sub'),
            numberInput(s.sample_rate, 8000, 48000, 1000, (v) => this.onChange({ sample_rate: v }), 'Hz')
        )}
        ${row(
            this._t('settings.audio.channels'),
            this._t('settings.audio.channels_sub'),
            numberInput(s.channels, 1, 2, 1, (v) => this.onChange({ channels: v }))
        )}
        ${row(
            this._t('settings.audio.chunk_duration'),
            this._t('settings.audio.chunk_duration_sub'),
            numberInput(s.chunk_duration_secs, 1, 60, 1, (v) => this.onChange({ chunk_duration_secs: v }), 's')
        )}
        ${row(
            this._t('settings.audio.mel_bins'),
            this._t('settings.audio.mel_bins_sub'),
            numberInput(s.mel_bins, 40, 256, 1, (v) => this.onChange({ mel_bins: v }))
        )}
        ${row(
            this._t('settings.audio.window_size'),
            this._t('settings.audio.window_size_sub'),
            numberInput(s.window_size, 128, 2048, 32, (v) => this.onChange({ window_size: v }))
        )}
        ${row(
            this._t('settings.audio.hop_size'),
            this._t('settings.audio.hop_size_sub'),
            numberInput(s.hop_size, 32, 1024, 16, (v) => this.onChange({ hop_size: v }))
        )}
      </div>
    `;
    }
}
