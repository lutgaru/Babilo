import { LitElement, html } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../../i18n';
import { applyTailwindToShadowRoot } from '../../lib/tailwind-styles';
import { row, sectionHeader, numberInput, select } from './settings-truncate';

@customElement('bbl-settings-model')
export class BblSettingsModel extends withI18n(LitElement) {
    @property({ type: Object }) data: Record<string, unknown> | null = null;
    @property({ type: Function }) onChange: (section: string, partial: Record<string, unknown>) => void = () => {};

    connectedCallback() {
        super.connectedCallback();
        if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
    }

    private _sd(section: string): Record<string, unknown> {
        return (this.data?.[section] as Record<string, unknown>) ?? {};
    }

    private _num(section: string, key: string): number {
        return (this._sd(section)?.[key] as number) ?? 0;
    }

    private _str(section: string, key: string): string {
        return String(this._sd(section)?.[key] ?? '');
    }

    render() {
        if (!this.data) return html``;
        return html`
      ${this._renderLlm()}
      ${this._renderInference()}
      ${this._renderAnalysis()}
    `;
    }

    private _rangeSlider(section: string, key: string, min: number, max: number, step: number) {
        const val = this._num(section, key);
        return html`
      <div class="flex items-center gap-2">
        <input type="range" min="${min}" max="${max}" step="${step}" .value="${String(val)}"
          @input=${(e: Event) => this.onChange(section, { [key]: Number((e.target as HTMLInputElement).value) })}
          class="appearance-none w-28 h-1 rounded bg-bbl-btn-bg border border-bbl-border
                 outline-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none
                 [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full
                 [&::-webkit-slider-thumb]:bg-bbl-accent2
                 [&::-webkit-slider-thumb]:shadow-[0_1px_4px_rgba(0,0,0,0.25)]">
        <span class="text-xs text-bbl-text-faint w-7 text-right">${val.toFixed(2)}</span>
      </div>`;
    }

    private _seedSelect(section: string) {
        return select(
            this._str(section, 'seed_option'),
            [
                { value: 'Random', label: this._t('settings.seed.random') },
                { value: 'Fixed', label: this._t('settings.seed.fixed') },
            ],
            (v) => this.onChange(section, { seed_option: v })
        );
    }

    private _renderLlm() {
        const s = 'llm';
        return html`
      ${sectionHeader(this._t('settings.llm.title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(this._t('settings.llm.context_size'), this._t('settings.llm.context_size_sub'), numberInput(this._num(s, 'context_size'), 512, 65536, 512, (v) => this.onChange(s, { context_size: v })))}
        ${row(this._t('settings.llm.batch_size'), this._t('settings.llm.batch_size_sub'), numberInput(this._num(s, 'batch_size'), 1, 8192, 128, (v) => this.onChange(s, { batch_size: v })))}
        ${row(this._t('settings.llm.ubatch_size'), this._t('settings.llm.ubatch_size_sub'), numberInput(this._num(s, 'ubatch_size'), 1, 2048, 64, (v) => this.onChange(s, { ubatch_size: v })))}
        ${row(this._t('settings.llm.n_gpu_layers'), this._t('settings.llm.n_gpu_layers_sub'), numberInput(this._num(s, 'n_gpu_layers'), 0, 200, 1, (v) => this.onChange(s, { n_gpu_layers: v })))}
        ${row(this._t('settings.llm.max_output_tokens'), this._t('settings.llm.max_output_tokens_sub'), numberInput(this._num(s, 'max_output_tokens'), 64, 65536, 512, (v) => this.onChange(s, { max_output_tokens: v })))}
      </div>`;
    }

    private _renderInference() {
        const s = 'inference';
        return html`
      ${sectionHeader(this._t('settings.inference.title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(this._t('settings.inference.temperature'), this._t('settings.inference.temperature_sub'), this._rangeSlider(s, 'temperature', 0, 2, 0.05))}
        ${row(this._t('settings.inference.top_p'), this._t('settings.inference.top_p_sub'), this._rangeSlider(s, 'top_p', 0, 1, 0.05))}
        ${row(this._t('settings.inference.top_k'), this._t('settings.inference.top_k_sub'), numberInput(this._num(s, 'top_k'), 1, 100, 1, (v) => this.onChange(s, { top_k: v })))}
        ${row(this._t('settings.inference.seed_option'), this._t('settings.inference.seed_option_sub'), this._seedSelect(s))}
        ${row(this._t('settings.inference.seed_value'), this._t('settings.inference.seed_value_sub'), numberInput(this._num(s, 'seed_value'), 0, 999999, 1, (v) => this.onChange(s, { seed_value: v })))}
      </div>`;
    }

    private _renderAnalysis() {
        const s = 'analysis';
        return html`
      ${sectionHeader(this._t('settings.analysis.title'))}
      <div class="flex flex-col px-2 gap-0.5">
        ${row(this._t('settings.analysis.context_size'), this._t('settings.analysis.context_size_sub'), numberInput(this._num(s, 'context_size'), 512, 16384, 256, (v) => this.onChange(s, { context_size: v })))}
        ${row(this._t('settings.analysis.max_output_tokens'), this._t('settings.analysis.max_output_tokens_sub'), numberInput(this._num(s, 'max_output_tokens'), 64, 16384, 64, (v) => this.onChange(s, { max_output_tokens: v })))}
        ${row(this._t('settings.inference.temperature'), this._t('settings.inference.temperature_sub'), this._rangeSlider(s, 'temperature', 0, 2, 0.05))}
        ${row(this._t('settings.inference.top_p'), this._t('settings.inference.top_p_sub'), this._rangeSlider(s, 'top_p', 0, 1, 0.05))}
        ${row(this._t('settings.inference.top_k'), this._t('settings.inference.top_k_sub'), numberInput(this._num(s, 'top_k'), 1, 100, 1, (v) => this.onChange(s, { top_k: v })))}
        ${row(this._t('settings.inference.seed_option'), this._t('settings.inference.seed_option_sub'), this._seedSelect(s))}
        ${row(this._t('settings.inference.seed_value'), this._t('settings.inference.seed_value_sub'), numberInput(this._num(s, 'seed_value'), 0, 999999, 1, (v) => this.onChange(s, { seed_value: v })))}
      </div>`;
    }
}
