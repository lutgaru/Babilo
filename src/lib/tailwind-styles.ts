// src/lib/tailwind-styles.ts
import tailwindStyles from '../tailwind.css?inline';

let _sheet: CSSStyleSheet | null = null;

function getSheet(): CSSStyleSheet {
  if (!_sheet) {
    _sheet = new CSSStyleSheet();
    _sheet.replaceSync(tailwindStyles);
  }
  return _sheet;
}

/**
 * Call this in connectedCallback() of every LitElement
 * to inject Tailwind into its shadow root.
 */
export function applyTailwindToShadowRoot(shadowRoot: ShadowRoot) {
  const sheet = getSheet();
  if (!shadowRoot.adoptedStyleSheets.includes(sheet)) {
    shadowRoot.adoptedStyleSheets = [...shadowRoot.adoptedStyleSheets, sheet];
  }
}

/** Legacy export — no longer needed, kept for compatibility */
export async function loadTailwindSheet(): Promise<void> {}