import { LitElement, html } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader, select } from './settings-truncate';
import type { GuiSettings } from '../../types/babilo';

@customElement('bbl-settings-appearance')
export class BblSettingsAppearance extends withI18n(LitElement) {
    @property({ type: Object }) data: GuiSettings | null = null;
    @property({ type: Function }) onChange: (partial: Partial<GuiSettings>) => void = () => {};

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    render() {
        if (!this.data) return html``;
        return html`
      ${sectionHeader(this._t('settings.appearance.title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(
            this._t('settings.appearance.theme'),
            this._t('settings.appearance.theme_sub'),
            select(
                this.data.theme,
                [
                    { value: 'light', label: this._t('settings.appearance.light') },
                    { value: 'dark', label: this._t('settings.appearance.dark') },
                ],
                (v) => this.onChange({ theme: v })
            )
        )}
      </div>
    `;
    }
}
