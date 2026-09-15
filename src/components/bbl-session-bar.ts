/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { SessionReport } from '../types/babilo';

@customElement('bbl-session-bar')
export class BblSessionBar extends withI18n(LitElement) {
  /** Latest session report packet from the backend (null = no turn yet) */
  @property({ type: Object }) report: SessionReport | null = null;

  static styles = css`:host { display: block; }`;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  private _pill(active: boolean, activeCls: string, idleCls: string, text: string) {
    return html`
      <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium border-[0.5px] ${active ? activeCls : idleCls}">
        <span class="w-1.5 h-1.5 rounded-full ${active ? 'bg-current' : 'bg-[var(--bbl-text-faint)]'}"></span>
        ${text}
      </span>
    `;
  }

  render() {
    const r = this.report;
    const pct = r ? Math.min(100, Math.max(0, r.context_percent)) : 0;
    const barColor = pct >= 90
      ? 'bg-[var(--bbl-accent)]'
      : pct >= 70
        ? 'bg-amber-500'
        : 'bg-[var(--bbl-accent2)]';

    return html`
      <div
        role="status"
        aria-label="Session report"
        class="bg-[var(--bbl-surface)] border-t-[0.5px] border-[var(--bbl-border)] px-4 py-2 flex items-center justify-center gap-3 flex-wrap text-center text-xs text-[var(--bbl-text-muted)]">

        <span class="inline-flex items-center gap-1.5 font-semibold text-[var(--bbl-text)]">
          <span class="text-[11px] uppercase tracking-[0.08em] text-[var(--bbl-text-faint)]">
            ${this._t('session_bar.turn')}
          </span>
          <span>${r ? r.turn : '—'}</span>
        </span>

        <span class="inline-flex items-center gap-2 min-w-[160px] flex-1 max-w-[280px]">
          <span class="text-[11px] uppercase tracking-[0.08em] text-[var(--bbl-text-faint)]">
            ${this._t('session_bar.context')}
          </span>
          <span class="flex-1 h-1.5 rounded-full bg-[var(--bbl-btn-bg)] border-[0.5px] border-[var(--bbl-border)] overflow-hidden">
            <span class="block h-full rounded-full ${barColor} transition-[width] duration-300" style="width: ${pct}%"></span>
          </span>
          <span class="tabular-nums font-medium text-[var(--bbl-text)]">
            ${r ? `${pct.toFixed(1)}%` : '—'}
          </span>
          <span class="tabular-nums text-[11px] text-[var(--bbl-text-faint)]">
            ${r ? `${r.context_used}/${r.context_total}` : ''}
          </span>
        </span>

        ${r ? this._pill(
          r.sys_prompt_injected,
          'bg-[var(--bbl-accent2)] border-[var(--bbl-accent2)] text-white',
          'bg-[var(--bbl-btn-bg)] border-[var(--bbl-border)] text-[var(--bbl-text-muted)]',
          `${this._t('session_bar.sys_injected')} ×${r.sys_prompt_injections}`
        ) : html`
          <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] border-[0.5px] bg-[var(--bbl-btn-bg)] border-[var(--bbl-border)] text-[var(--bbl-text-faint)]">
            ${this._t('session_bar.sys_injected')} —
          </span>
        `}

        ${r ? this._pill(
          r.context_cleaned,
          'bg-[var(--bbl-accent-dim)] border-[var(--bbl-accent-ring)] text-[var(--bbl-accent)]',
          'bg-[var(--bbl-btn-bg)] border-[var(--bbl-border)] text-[var(--bbl-text-muted)]',
          r.context_cleaned ? this._t('session_bar.cleaned') : this._t('session_bar.sys_kept')
        ) : html`
          <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] border-[0.5px] bg-[var(--bbl-btn-bg)] border-[var(--bbl-border)] text-[var(--bbl-text-faint)]">
            ${this._t('session_bar.sys_clean')} —
          </span>
        `}
      </div>
    `;
  }
}
