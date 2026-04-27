/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export const listAudioDevices = ()                        => tauriInvoke('list_audio_devices');
export const startListening   = (deviceName: string|null) => tauriInvoke('start_listening', { deviceName });
export const stopAndProcess   = (prompt: string)          => tauriInvoke('stop_and_process', { prompt });
export const synthesize       = (text: string)            => tauriInvoke('synthesize', { text, voice: 'F1' });
export const testInference    = (testPrompt: string)      => tauriInvoke('test_inference', { testPrompt });