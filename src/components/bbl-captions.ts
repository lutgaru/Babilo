/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';

@customElement('bbl-captions')
export class BblCaptions extends withI18n(LitElement) {
  /** Latest model response text to display. Empty hides the caption. */
  @property({ type: String })
  text = '';

  /** Master switch from Settings → Appearance. */
  @property({ type: Boolean })
  visible = true;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) applyTailwindToShadowRoot(this.shadowRoot);
  }

  render() {
    const caption = (this.text ?? '').trim();
    if (!this.visible || !caption) return html``;

    return html`
      <div class="pointer-events-none absolute inset-x-0 bottom-0 z-10
                  flex justify-center px-4 pb-4">
        <p role="status" aria-live="polite"
           aria-label="${this._t('captions.label')}"
           class="max-w-[min(640px,90%)] rounded-lg px-4 py-2
                  text-center text-sm leading-relaxed text-white
                  bg-black/50">
          ${caption}
        </p>
      </div>
    `;
  }
}
