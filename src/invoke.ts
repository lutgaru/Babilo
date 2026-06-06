/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { AppSettings, AudioDevice, ModeFileInfo, SessionInfo, SessionSummary } from './types/babilo';
import * as mocks from './mocks/babiloMocks';
import { simulateRustStream } from './mocks/babiloMocks';

const isTauri = (): boolean => !!(window as any).__TAURI_INTERNALS__;
const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const listAudioDevices = async (): Promise<AudioDevice[]> => {
  if (isTauri()) return tauriInvoke('list_audio_devices');
  await delay(300);
  return mocks.mockAudioDevices;
};

export const startListening = async (deviceName: string | null): Promise<void> => {
  if (isTauri()) return tauriInvoke('start_listening', { deviceName });
  console.log(`[Babilo Mock] Capturando en: ${deviceName ?? 'Default'}`);
  return;
};

export const stopAndProcessStreaming = async (prompt: string): Promise<any> => {
  if (isTauri()) return tauriInvoke('stop_and_process_streaming', { prompt });
  simulateRustStream(prompt);
  return { success: true };
};

export const processTextStreaming = async (prompt: string): Promise<any> => {
  if (isTauri()) return tauriInvoke('process_text_streaming', { prompt });
  console.log(`[Babilo Mock] processTextStreaming con prompt: "${prompt}"`);
  await delay(500);
  return { success: true };
};

export const endSession = async (): Promise<SessionSummary> => {
  if (isTauri()) return tauriInvoke('end_session');
  await delay(600);
  return mocks.mockSessionSummary("10", "mock-session-mode");
};

export const listModes = async (): Promise<ModeFileInfo[]> => {
  if (isTauri()) return tauriInvoke('get_list_modes');
  await delay(400);
  return mocks.mockModes;
};

export const startSession = async (path: string): Promise<SessionInfo> => {
  if (isTauri()) return tauriInvoke('start_session', { path });
  await delay(500);
  return mocks.mockSessionStart(path);
};

export const loadSettings = async (): Promise<AppSettings> => {
  if (isTauri()) return tauriInvoke('load_settings');
  await delay(200);
  return mocks.mockSettings;
};

export const saveSettings = async (settings: AppSettings): Promise<void> => {
  if (isTauri()) return tauriInvoke('save_settings', { settings });
  console.log('[Babilo Mock] Settings saved:', settings);
};

export const resetContext = async (): Promise<void> => {
  if (isTauri()) { await tauriInvoke('reset_conversation'); return; }
  console.log('[Babilo Mock] Context reset');
  await delay(300);
};