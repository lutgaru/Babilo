/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

export type Locale = 'en' | 'es';

export const translations = {
  en: {
    // Common
    'common.loading': 'Loading...',
    'common.error': 'Error',
    'common.default': 'Default',
    'common.refresh': 'Refresh',
    'common.close': 'Close',
    'common.send': 'Send',
    'common.assistant': 'Assistant',

    // States
    'state.idle': 'Ready',
    'state.listening': 'Listening...',
    'state.thinking': 'Thinking...',
    'state.processing': 'Processing...',
    'state.speaking': 'Speaking...',

    // Avatar
    'avatar.label': 'AI Assistant avatar',

    // Config List
    'config.title': 'Babilo',
    'config.subtitle': 'Choose a mode to practice',
    'config.no_modes': 'No modes available',
    'config.starting': 'Starting...',
    'config.cap.audio': 'Audio input',
    'config.cap.text': 'Text input',
    'config.cap.llm_first': 'AI speaks first',

    // Controls
    'controls.mute': 'Reset conversation',
    'controls.volume': 'Volume',
    'controls.record_toggle': 'Record / Stop',
    'controls.transcript_show': 'Show transcript',
    'controls.transcript_hide': 'Hide transcript',
    'controls.transcript_panel_show': 'Show transcript panel',
    'controls.transcript_panel_hide': 'Hide transcript panel',
    'controls.hangup': 'End call',

    // Mic Panel
    'mic.label': 'Microphone',
    'mic.refresh': 'Refresh devices',

    // Transcript
    'transcript.empty': 'The conversation will appear here...',
    'transcript.analysis': 'Analysis',
    'transcript.no_corrections': '✓ No corrections needed',

    // Top Bar
    'topbar.active_session': 'Active session',
    'topbar.end_session': 'End session',
    'topbar.settings': 'Settings',

    // Splash
    'splash.brand': 'BABILO',
    'splash.tagline': 'Local-First Language AI',
    'splash.init': 'Initializing language engines...',
    'splash.sync_vulkan': 'Synchronizing Vulkan environments...',
    'splash.load_vram': 'Instantiating network weights in VRAM...',
    'splash.error_hint': 'Check system logs or hardware configuration.',
    'splash.license': 'GPL-3.0-or-later • Copyright (C) 2026 Lutgaru',

    //settings
    'settings.title': 'Settings',

    'settings.nav.audio': 'Audio',
    'settings.audio.input_title': 'Audio Input',
    'settings.audio.microphone': 'Microphone',
    'settings.audio.microphone_sub': 'Select your input device',
    'settings.audio.default': 'Default',
    'settings.audio.output_title': 'Audio Output',
    'settings.audio.output_volume': 'Volume',
    'settings.audio.output_volume_sub': 'Adjust the output volume',

    'settings.nav.language': 'Language',
    'settings.language.interface_title': 'Language Settings',
    'settings.language.ui_language': 'Interface Language',
    'settings.language.ui_language_sub': 'Choose the language for the user interface',

    'settings.close': 'Close',
  },

  es: {
    // Common
    'common.loading': 'Cargando...',
    'common.error': 'Error',
    'common.default': 'Predeterminado',
    'common.refresh': 'Actualizar',
    'common.close': 'Cerrar',
    'common.send': 'Enviar',
    'common.assistant': 'Asistente',

    // States
    'state.idle': 'Listo',
    'state.listening': 'Escuchando...',
    'state.thinking': 'Pensando...',
    'state.processing': 'Procesando...',
    'state.speaking': 'Hablando...',

    // Avatar
    'avatar.label': 'Avatar del Asistente de IA',

    // Config List
    'config.title': 'Babilo',
    'config.subtitle': 'Elige un modo para practicar',
    'config.no_modes': 'No hay modos disponibles',
    'config.starting': 'Iniciando...',
    'config.cap.audio': 'Entrada de audio',
    'config.cap.text': 'Entrada de texto',
    'config.cap.llm_first': 'IA habla primero',

    // Controls
    'controls.mute': 'Reiniciar conversación',
    'controls.volume': 'Volumen',
    'controls.record_toggle': 'Grabar / Detener',
    'controls.transcript_show': 'Mostrar transcripción',
    'controls.transcript_hide': 'Ocultar transcripción',
    'controls.transcript_panel_show': 'Mostrar panel de transcripción',
    'controls.transcript_panel_hide': 'Ocultar panel de transcripción',
    'controls.hangup': 'Colgar',

    // Mic Panel
    'mic.label': 'Micrófono',
    'mic.refresh': 'Actualizar dispositivos',

    // Transcript
    'transcript.empty': 'La conversación aparecerá aquí...',
    'transcript.analysis': 'Análisis',
    'transcript.no_corrections': '✓ Sin correcciones necesarias',

    // Top Bar
    'topbar.active_session': 'Sesión activa',
    'topbar.end_session': 'Terminar sesión',
    'topbar.settings': 'Configuración',
    // Splash
    'splash.brand': 'BABILO',
    'splash.tagline': 'IA de Lenguaje Local-First',
    'splash.init': 'Inicializando motores de lenguaje...',
    'splash.sync_vulkan': 'Sincronizando entornos Vulkan...',
    'splash.load_vram': 'Instanciando pesos de red en VRAM...',
    'splash.error_hint': 'Revisa los logs del sistema o la configuración de hardware.',
    'splash.license': 'GPL-3.0-or-later • Copyright (C) 2026 Lutgaru',

    //settings
    'settings.title': 'Configuración',

    'settings.nav.audio': 'Audio',
    'settings.audio.input_title': 'Entrada de audio',
    'settings.audio.microphone': 'Micrófono',
    'settings.audio.microphone_sub': 'Selecciona tu dispositivo de entrada',
    'settings.audio.default': 'Predeterminado',
    'settings.audio.output_title': 'Salida de audio',

    'settings.audio.output_volume': 'Volumen',
    'settings.audio.output_volume_sub': 'Ajusta el volumen de salida',

    'settings.nav.language': 'Idioma',
    'settings.language.interface_title': 'Configuración de idioma',
    'settings.language.ui_language': 'Idioma de la interfaz',
    'settings.language.ui_language_sub': 'Elige el idioma para la interfaz de usuario',

    'settings.close': 'Cerrar',
  },
} as const;

export type TranslationKey = keyof typeof translations.en;