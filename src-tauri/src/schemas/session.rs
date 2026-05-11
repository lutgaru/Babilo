/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */
 
//! Session structs that travel between Rust and the frontend.
//!
//! The logic that builds these structs lives in `session/mod.rs`.
//! This module only re-exports them so the frontend can find them
//! always under `crate::schemas::*`.
 
pub use crate::session::{SessionCaps, SessionInfo, SessionSummary};
 
