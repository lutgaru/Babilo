/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { AudioDevice, ModeFileInfo, SessionInfo, SessionSummary } from './types/babilo';

export const listAudioDevices = (): Promise<AudioDevice[]> => tauriInvoke('list_audio_devices');
export const startListening = (deviceName: string | null) => tauriInvoke('start_listening', { deviceName });
export const stopAndProcessStreaming = (prompt: string) => tauriInvoke('stop_and_process_streaming', { prompt });
export const processTextStreaming = (prompt: string) => tauriInvoke('process_text_streaming', { prompt });
export const endSession = (): Promise<SessionSummary> => tauriInvoke('end_session');
export const listModes = (): Promise<ModeFileInfo[]> => tauriInvoke('get_list_modes');
export const startSession = (path: string): Promise<SessionInfo> => tauriInvoke('start_session', { path });