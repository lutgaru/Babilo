/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { LitElement, html, css } from 'lit';
import { property, customElement } from 'lit/decorators.js';
import { withI18n } from '../i18n';
import { applyTailwindToShadowRoot } from '../lib/tailwind-styles';
import type { AIState } from '../types/babilo';

@customElement('bbl-avatar')
export class BblAvatar extends withI18n(LitElement) {
  @property({ type: String })
  state: AIState = 'idle';

  @property({ type: String })
  label = 'AI';

  // ── Computed helpers ──
  private get _isPulsing(): boolean {
    return this.state === 'speaking';
  }

  static styles = css`
    :host {
      display: block;
    }

    /* ── Keyframes ── */
    
    /* Sutil pulse para estado idle (siempre activo, base) */
    @keyframes pulse-idle {
      0%, 100% { 
        transform: scale(1); 
        opacity: 0.3; 
      }
      50% { 
        transform: scale(1.03); 
        opacity: 0.5; 
      }
    }
    
    /* Pulse intenso para listening/speaking (override) */
    @keyframes pulse-a {
      0%   { transform: scale(1);    opacity: 0.7; }
      100% { transform: scale(1.15); opacity: 0;   }
    }
    @keyframes pulse-b {
      0%   { transform: scale(1);    opacity: 0.5; }
      100% { transform: scale(1.12); opacity: 0;   }
    }

    /* ── Ring base styles ── */
    .ring {
      position: absolute;
      border-radius: 9999px;
      pointer-events: none;
    }

    .ring-c {
      width: 98px;
      height: 98px;
      border: 1px solid var(--bbl-ring-c);
      background: var(--bbl-ring-c);
      /* Ring-c siempre estático (el más interno) */
    }

    .ring-b {
      width: 128px;
      height: 128px;
      border: 1px solid var(--bbl-ring-b);
      /* Idle pulse por defecto */
      animation: pulse-idle 3s ease-in-out infinite;
    }

    .ring-a {
      width: 160px;
      height: 160px;
      border: 1px solid var(--bbl-ring-a);
      /* Idle pulse por defecto, más sutil */
      animation: pulse-idle 4s ease-in-out infinite;
    }

    /* ── Pulsing animations (override cuando data-pulsing="true") ── */
    [data-pulsing="true"] .ring-a {
      animation: pulse-a 1.6s ease-out infinite;
    }
    [data-pulsing="true"] .ring-b {
      animation: pulse-b 1.6s 0.3s ease-out infinite;
    }

    /* ── Avatar circle ── */
    .avatar {
      position: relative;
      z-index: 2;
      width: 76px;
      height: 76px;
      border-radius: 9999px;
      background: linear-gradient(135deg, var(--bbl-accent2), var(--bbl-accent));
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 22px;
      font-weight: 600;
      color: white;
      border: 1px solid rgba(255, 255, 255, 0.15);
      letter-spacing: 0.04em;
      user-select: none;
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    if (this.shadowRoot) {
      applyTailwindToShadowRoot(this.shadowRoot);
    }
  }

  render() {
    return html`
      <div 
        class="relative w-40 h-40 flex items-center justify-center"
        data-pulsing=${this._isPulsing}
        aria-label="${this._t('avatar.label')}">
        
        <!-- ring-c: filled innermost (siempre estático) -->
        <div class="ring ring-c"></div>
        
        <!-- ring-b: con pulse sutil o intenso según estado -->
        <div class="ring ring-b"></div>
        
        <!-- ring-a: con pulse sutil o intenso según estado -->
        <div class="ring ring-a"></div>
        
        <!-- avatar circle -->
        <div class="avatar">
          ${this.label}
        </div>
      </div>
    `;
  }
}