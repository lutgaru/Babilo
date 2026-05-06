// tailwind.config.js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{ts,html}',
  ],
  theme: {
    extend: {
      colors: {
        'bbl-bg':          'var(--bbl-bg)',
        'bbl-surface':     'var(--bbl-surface)',
        'bbl-surface-alt': 'var(--bbl-surface-alt)',
        'bbl-panel':       'var(--bbl-panel)',
        'bbl-text':        'var(--bbl-text)',
        'bbl-text-muted':  'var(--bbl-text-muted)',
        'bbl-text-faint':  'var(--bbl-text-faint)',
        'bbl-accent':      'var(--bbl-accent)',
        'bbl-accent2':     'var(--bbl-accent2)',
        'bbl-border':      'var(--bbl-border)',
        'bbl-border-md':   'var(--bbl-border-md)',
        'bbl-btn-bg':      'var(--bbl-btn-bg)',
        'bbl-btn-hover':   'var(--bbl-btn-hover)',
        'bbl-ring-a':      'var(--bbl-ring-a)',
        'bbl-ring-b':      'var(--bbl-ring-b)',
        'bbl-ring-c':      'var(--bbl-ring-c)',
        'bbl-status-idle': 'var(--bbl-status-idle)',
        'bbl-status-live': 'var(--bbl-status-live)',
      },
      borderRadius: {
        'bbl-sm':   'var(--bbl-radius-sm)',
        'bbl-md':   'var(--bbl-radius-md)',
        'bbl-lg':   'var(--bbl-radius-lg)',
        'bbl-pill': 'var(--bbl-radius-pill)',
      },
    },
  },
  plugins: [],
}