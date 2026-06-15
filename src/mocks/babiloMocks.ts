/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * Mock data aligned with the official application types
 */

import {
    AppSettings,
    AudioDevice,
    ModeFileInfo,
    SessionInfo,
    SessionSummary,
    BabiloAnalysis,
    TranscriptMessage,
    StreamEvent
} from '../types/babilo';

// 1. Audio Devices
export const mockAudioDevices: AudioDevice[] = [
    { id: 'default', name: 'System Microphone (Default)' },
    { id: 'hw_0_0', name: 'ALSA: USB Audio Device PCM (Hardware)' },
    { id: 'virtual_mic', name: 'PipeWire Virtual Source (Babilo Capture)' }
];

// 2. Available Learning Modes (Loaded from fictional .babilo.json files)
export const mockModes: ModeFileInfo[] = [
    {
        path: 'modes/interview.babilo.json',
        name: 'Professional Interview Mode',
        description: 'Simulates a strict technical interview. The LLM initiates the conversation.',
        caps: { accepts_audio: true, accepts_text: true, llm_initiates: true }
    },
    {
        path: 'modes/casual.babilo.json',
        name: 'Free Casual Chat',
        description: 'Fluid conversation with friendly grammar correction at user request.',
        caps: { accepts_audio: true, accepts_text: true, llm_initiates: false }
    },
    {
        path: 'modes/listening.babilo.json',
        name: 'Comprehensive Listening Mode',
        description: 'Listens to AI-generated text and answers specific comprehension questions.',
        caps: { accepts_audio: false, accepts_text: true, llm_initiates: true }
    }
];

// 3. Simulated Session Start
export const mockSessionStart = (path: string): SessionInfo => {
    const selectedMode = mockModes.find(m => m.path === path) || mockModes[1];

    return {
        session_id: `bbl_sess_${Math.random().toString(36).substring(2, 9)}`,
        mode_id: selectedMode.path.replace('modes/', '').replace('.babilo.json', ''),
        mode_name: selectedMode.name,
        caps: selectedMode.caps,
        opening_line: selectedMode.caps.llm_initiates
            ? 'Welcome to your technical review. Can you introduce yourself and tell me about your experience with embedded systems?'
            : null
    };
};

// 4. Babilo Analysis and Inference (What your orchestrator grinds through)
export const mockAnalysisExamples: BabiloAnalysis[] = [
    {
        transcription: "I wants to learning English for build a native application.",
        response: "That is an awesome goal! Native applications are extremely fast. Just a quick note: we say 'I want to learn' instead of 'wants to learning'. What programming language are you planning to use?",
        corrections: [
            { original: "I wants", fixed: "I want", reason: "First person singular ('I') does not add an 's' to the verb in present tense." },
            { original: "to learning", fixed: "to learn", reason: "After 'want to', you must use the infinitive base form of the verb." },
            { original: "for build", fixed: "to build", reason: "To express purpose, we generally use 'to + infinitive' or 'for + gerund (-ing)', but 'to build' sounds much more natural here." }
        ],
        score: 68,
        next_step_hint: "Try answering with the specific language, like: 'I am planning to use Rust.'"
    },
    {
        transcription: "I am planning to use Rust because it have good performance.",
        response: "Excellent choice! Rust offers incredible speed and safety. Watch out for the subject-verb agreement with 'it'. It takes 'has' instead of 'have'. Are you planning to build a desktop app or a CLI tool?",
        corrections: [
            { original: "it have", fixed: "it has", reason: "The pronoun 'it' is third person singular, so it requires the conjugated form 'has'." }
        ],
        score: 88,
        next_step_hint: "You can say: 'I want to build a desktop application using Tauri.'"
    }
];



// 5. Message History (Transcripts)
export const mockTranscriptHistory = (): TranscriptMessage[] => [
    {
        analysis: mockAnalysisExamples[0],
        timestamp: Date.now() - 60000
    },
    {
        analysis: mockAnalysisExamples[1],
        timestamp: Date.now() - 30000
    }
];

// 6. End of Session Summary
export const mockSessionSummary = (sessionId: string, modeName: string): SessionSummary => ({
    session_id: sessionId,
    mode_name: modeName,
    turns: 5,
    average_score: 78
});

// 7. Default Settings (mirrors Rust PersistentSettings::default())
export const mockSettings: AppSettings = {
    audio: {
        sample_rate: 16000,
        channels: 1,
        chunk_duration_secs: 30,
        mel_bins: 128,
        window_size: 320,
        hop_size: 160,
    },
    llm: {
        context_size: 4096,
        batch_size: 2048,
        ubatch_size: 512,
        n_gpu_layers: 99,
        max_output_tokens: 10000,
    },
    inference: {
        temperature: 0.7,
        top_p: 0.9,
        top_k: 40,
        seed_option: 'Random',
        seed_value: 7,
    },
    tts: null,
    gui: {
        theme: 'light',
        language: 'en',
    },
};

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** Emit a mock ai-state event (mirrors backend babilo://ai-state) */
function emitAiState(state: 'thinking' | 'speaking' | 'idle') {
    window.dispatchEvent(new CustomEvent('babilo://ai-state', { detail: { payload: state } }));
}

/**
 * Simula la emisión secuencial de eventos simulando el backend de Rust + Vulkan
 */
export const simulateRustStream = async (prompt: string): Promise<void> => {
    console.log(`[Babilo Engine Mock] Analizando prompt: "${prompt}"...`);

    emitAiState('thinking');
    await delay(100);

    emitAiState('speaking');
    const sentinelEvent: StreamEvent = {
        type: 'sentinel_reached'
    };
    window.dispatchEvent(new CustomEvent('babilo://stream', { detail: { payload: sentinelEvent } }));

    await delay(1000);

    emitAiState('idle');
    const analysisEvent: StreamEvent = {
        type: 'analysis',
        data: mockAnalysisExamples[0]
    };
    window.dispatchEvent(new CustomEvent('babilo://stream', { detail: { payload: analysisEvent } }));
};
