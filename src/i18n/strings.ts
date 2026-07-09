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
    'settings.audio.advanced_title': 'Audio Processing',
    'settings.audio.sample_rate': 'Sample Rate',
    'settings.audio.sample_rate_sub': 'Audio sample rate in Hz',
    'settings.audio.channels': 'Channels',
    'settings.audio.channels_sub': 'Number of audio channels',
    'settings.audio.chunk_duration': 'Chunk Duration',
    'settings.audio.chunk_duration_sub': 'Duration of each audio chunk',
    'settings.audio.mel_bins': 'Mel Bins',
    'settings.audio.mel_bins_sub': 'Number of mel filterbank bins',
    'settings.audio.window_size': 'Window Size',
    'settings.audio.window_size_sub': 'FFT window size in samples',
    'settings.audio.hop_size': 'Hop Size',
    'settings.audio.hop_size_sub': 'Window hop size in samples',

    'settings.nav.language': 'Language',
    'settings.language.interface_title': 'Language Settings',
    'settings.language.ui_language': 'Interface Language',
    'settings.language.ui_language_sub': 'Choose the language for the user interface',

    'settings.nav.model': 'Model',
    'settings.nav.llm': 'LLM',
    'settings.llm.title': 'LLM Configuration',
    'settings.llm.context_size': 'Context Size',
    'settings.llm.context_size_sub': 'Maximum tokens in context window',
    'settings.llm.batch_size': 'Batch Size',
    'settings.llm.batch_size_sub': 'Tokens processed per batch',
    'settings.llm.ubatch_size': 'Micro Batch Size',
    'settings.llm.ubatch_size_sub': 'Tokens per micro batch',
    'settings.llm.n_gpu_layers': 'GPU Layers',
    'settings.llm.n_gpu_layers_sub': 'Number of layers offloaded to GPU',
    'settings.llm.max_output_tokens': 'Max Output Tokens',
    'settings.llm.max_output_tokens_sub': 'Maximum tokens in model response',

    'settings.nav.inference': 'Inference',
    'settings.inference.title': 'Inference Configuration',
    'settings.inference.temperature': 'Temperature',
    'settings.inference.temperature_sub': 'Controls randomness (0 = deterministic)',
    'settings.inference.top_p': 'Top P',
    'settings.inference.top_p_sub': 'Nucleus sampling threshold',
    'settings.inference.top_k': 'Top K',
    'settings.inference.top_k_sub': 'Top-K sampling candidates',
    'settings.inference.seed_option': 'Seed Mode',
    'settings.inference.seed_option_sub': 'Random or fixed seed',
    'settings.inference.seed_value': 'Seed Value',
    'settings.inference.seed_value_sub': 'Fixed seed for reproducibility',

    'settings.nav.analysis': 'Analysis',
    'settings.analysis.title': 'Analysis Configuration',
    'settings.analysis.context_size': 'Context Size',
    'settings.analysis.context_size_sub': 'Analysis context window size',
    'settings.analysis.max_output_tokens': 'Max Output Tokens',
    'settings.analysis.max_output_tokens_sub': 'Maximum tokens in analysis output',

    'settings.nav.tts': 'TTS',
    'settings.tts.title': 'Text-to-Speech',
    'settings.tts.ae_sample_rate': 'AE Sample Rate',
    'settings.tts.ae_sample_rate_sub': 'Audio encoder sample rate',
    'settings.tts.ae_chunk_size': 'AE Base Chunk Size',
    'settings.tts.ae_chunk_size_sub': 'Audio encoder base chunk size',
    'settings.tts.ttl_compress': 'TTL Compress Factor',
    'settings.tts.ttl_compress_sub': 'Text-to-latent compression factor',
    'settings.tts.ttl_latent_dim': 'TTL Latent Dim',
    'settings.tts.ttl_latent_dim_sub': 'Text-to-latent latent dimension',

    'settings.nav.appearance': 'Appearance',
    'settings.appearance.title': 'Appearance',
    'settings.appearance.theme': 'Theme',
    'settings.appearance.theme_sub': 'Choose light or dark theme',
    'settings.appearance.light': 'Light',
    'settings.appearance.dark': 'Dark',

    'settings.seed.random': 'Random',
    'settings.seed.fixed': 'Fixed',

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
    'settings.audio.advanced_title': 'Procesamiento de Audio',
    'settings.audio.sample_rate': 'Frec. de Muestreo',
    'settings.audio.sample_rate_sub': 'Frecuencia de muestreo de audio en Hz',
    'settings.audio.channels': 'Canales',
    'settings.audio.channels_sub': 'Número de canales de audio',
    'settings.audio.chunk_duration': 'Duración de Chunk',
    'settings.audio.chunk_duration_sub': 'Duración de cada fragmento de audio',
    'settings.audio.mel_bins': 'Bins Mel',
    'settings.audio.mel_bins_sub': 'Número de bins del banco de filtros mel',
    'settings.audio.window_size': 'Tamaño de Ventana',
    'settings.audio.window_size_sub': 'Tamaño de ventana FFT en muestras',
    'settings.audio.hop_size': 'Tamaño de Salto',
    'settings.audio.hop_size_sub': 'Tamaño de salto de ventana en muestras',

    'settings.nav.language': 'Idioma',
    'settings.language.interface_title': 'Configuración de idioma',
    'settings.language.ui_language': 'Idioma de la interfaz',
    'settings.language.ui_language_sub': 'Elige el idioma para la interfaz de usuario',

    'settings.nav.model': 'Modelo',
    'settings.nav.llm': 'LLM',
    'settings.llm.title': 'Configuración del LLM',
    'settings.llm.context_size': 'Tamaño de Contexto',
    'settings.llm.context_size_sub': 'Máximo de tokens en la ventana de contexto',
    'settings.llm.batch_size': 'Tamaño de Lote',
    'settings.llm.batch_size_sub': 'Tokens procesados por lote',
    'settings.llm.ubatch_size': 'Micro Lote',
    'settings.llm.ubatch_size_sub': 'Tokens por micro lote',
    'settings.llm.n_gpu_layers': 'Capas GPU',
    'settings.llm.n_gpu_layers_sub': 'Capas descargadas a la GPU',
    'settings.llm.max_output_tokens': 'Tokens Máximos de Salida',
    'settings.llm.max_output_tokens_sub': 'Máximo de tokens en respuesta del modelo',

    'settings.nav.inference': 'Inferencia',
    'settings.inference.title': 'Configuración de Inferencia',
    'settings.inference.temperature': 'Temperatura',
    'settings.inference.temperature_sub': 'Controla la aleatoriedad (0 = determinista)',
    'settings.inference.top_p': 'Top P',
    'settings.inference.top_p_sub': 'Umbral de muestreo nucleico',
    'settings.inference.top_k': 'Top K',
    'settings.inference.top_k_sub': 'Candidatos de muestreo Top-K',
    'settings.inference.seed_option': 'Modo Semilla',
    'settings.inference.seed_option_sub': 'Semilla aleatoria o fija',
    'settings.inference.seed_value': 'Valor de Semilla',
    'settings.inference.seed_value_sub': 'Semilla fija para reproducibilidad',

    'settings.nav.analysis': 'Análisis',
    'settings.analysis.title': 'Configuración de Análisis',
    'settings.analysis.context_size': 'Tamaño de Contexto',
    'settings.analysis.context_size_sub': 'Tamaño de la ventana de contexto de análisis',
    'settings.analysis.max_output_tokens': 'Tokens. Máximos de Salida',
    'settings.analysis.max_output_tokens_sub': 'Máximo de tokens en salida de análisis',

    'settings.nav.tts': 'TTS',
    'settings.tts.title': 'Texto a Voz',
    'settings.tts.ae_sample_rate': 'AE Frec. de Muestreo',
    'settings.tts.ae_sample_rate_sub': 'Frecuencia de muestreo del codificador de audio',
    'settings.tts.ae_chunk_size': 'AE Tamaño Base de Chunk',
    'settings.tts.ae_chunk_size_sub': 'Tamaño base de chunk del codificador de audio',
    'settings.tts.ttl_compress': 'TTL Factor de Compresión',
    'settings.tts.ttl_compress_sub': 'Factor de compresión texto-a-latente',
    'settings.tts.ttl_latent_dim': 'TTL Dimensión Latente',
    'settings.tts.ttl_latent_dim_sub': 'Dimensión latente de texto-a-latente',

    'settings.nav.appearance': 'Apariencia',
    'settings.appearance.title': 'Apariencia',
    'settings.appearance.theme': 'Tema',
    'settings.appearance.theme_sub': 'Elige tema claro u oscuro',
    'settings.appearance.light': 'Claro',
    'settings.appearance.dark': 'Oscuro',

    'settings.seed.random': 'Aleatorio',
    'settings.seed.fixed': 'Fijo',

    'settings.close': 'Cerrar',
  },
} as const;

export type TranslationKey = keyof typeof translations.en;