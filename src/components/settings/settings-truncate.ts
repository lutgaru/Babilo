import { html } from 'lit';
import type { TemplateResult } from 'lit';

export function row(label: string, sublabel: string, control: TemplateResult): TemplateResult {
    return html`
      <div class="flex items-center justify-between
                  px-5 py-3.5 rounded-xl gap-4
                  transition-[background] duration-150 hover:bg-bbl-btn-hover">
        <div class="flex flex-col gap-0.5 min-w-0">
          <span class="text-sm text-bbl-text leading-snug">${label}</span>
          <span class="text-xs text-bbl-text-faint leading-snug truncate">${sublabel}</span>
        </div>
        <div class="flex-shrink-0">${control}</div>
      </div>
    `;
}

export function sectionHeader(title: string): TemplateResult {
    return html`
      <p class="px-5 pt-5 pb-1 text-[10px] font-semibold uppercase tracking-[0.1em]
                text-bbl-text-faint">
        ${title}
      </p>
    `;
}

export function numberInput(value: number, min: number, max: number, step: number, onChange: (v: number) => void, unit?: string): TemplateResult {
    return html`
      <div class="flex items-center gap-2">
        <input type="number" min="${min}" max="${max}" step="${step}"
          .value="${String(value)}"
          @change=${(e: Event) => onChange(Number((e.target as HTMLInputElement).value))}
          class="w-20 bg-bbl-surface border border-bbl-border text-bbl-text
                 rounded-bbl-sm px-2.5 py-1.5 text-xs text-right
                 outline-none focus:border-bbl-accent2
                 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none">
        ${unit ? html`<span class="text-xs text-bbl-text-faint">${unit}</span>` : ''}
      </div>
    `;
}

export function slider(value: number, min: number, max: number, step: number, onChange: (v: number) => void): TemplateResult {
    return html`
      <div class="w-28">
        <input type="range" min="${min}" max="${max}" step="${step}" .value="${String(value)}"
          @input=${(e: Event) => onChange(Number((e.target as HTMLInputElement).value))}
          class="appearance-none w-full h-1 rounded bg-bbl-btn-bg border border-bbl-border
                 outline-none cursor-pointer
                 [&::-webkit-slider-thumb]:appearance-none
                 [&::-webkit-slider-thumb]:w-4
                 [&::-webkit-slider-thumb]:h-4
                 [&::-webkit-slider-thumb]:rounded-full
                 [&::-webkit-slider-thumb]:bg-bbl-accent2
                 [&::-webkit-slider-thumb]:shadow-[0_1px_4px_rgba(0,0,0,0.25)]
                 [&::-webkit-slider-thumb]:transition-transform
                 [&::-webkit-slider-thumb]:duration-150
                 [&::-webkit-slider-thumb]:hover:scale-110">
      </div>
    `;
}

export function select(value: string, options: Array<{ value: string; label: string }>, onChange: (v: string) => void): TemplateResult {
    return html`
      <div class="relative">
        <select
          @change=${(e: Event) => onChange((e.target as HTMLSelectElement).value)}
          class="appearance-none bg-bbl-surface border border-bbl-border text-bbl-text
                 rounded-bbl-sm px-2.5 py-1.5 text-xs cursor-pointer outline-none
                 focus:border-bbl-accent2 pr-7">
          ${options.map(opt => html`
            <option value="${opt.value}" ?selected=${value === opt.value}>${opt.label}</option>
          `)}
        </select>
        <svg class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2
                    w-3 h-3 text-bbl-text-muted"
             viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 9l6 6 6-6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    `;
}
