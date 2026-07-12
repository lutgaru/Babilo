import { LitElement, html } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader, numberInput } from './settings-truncate';
import type { TtsSettings } from '../../types/babilo';
import '../bbl-mic-panel';

@customElement('bbl-settings-audio')
export class BblSettingsAudio extends withI18n(LitElement) {
    @property({ type: Object }) data: TtsSettings | null = null;
    @property({ type: Function }) onChange: (partial: Partial<TtsSettings>) => void = () => {};

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
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
