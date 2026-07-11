import { LitElement, html } from 'lit';
import { customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader } from './settings-truncate';
import '../bbl-mic-panel';

@customElement('bbl-settings-audio')
export class BblSettingsAudio extends withI18n(LitElement) {
    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    render() {
        return html`
      ${sectionHeader(this._t('settings.audio.input_title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(
            this._t('settings.audio.microphone'),
            this._t('settings.audio.microphone_sub'),
            html`<bbl-mic-panel></bbl-mic-panel>`
        )}
      </div>
    `;
    }
}
