/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Módulo de audio: captura y procesamiento

pub mod capture;
pub mod processor;

pub use capture::{AudioCapture, AudioDeviceInfo, list_input_devices};
pub use processor::MelPreprocessor;

// Re-export del handle seguro para streams
// pub use crate::state::AudioStreamHandle;