/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

 
//! Utilidades generales

use std::path::{Path, PathBuf};

/// Obtener ruta al directorio de modelos
pub fn models_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Project root not found")
        .join("models")
}

/// Obtener ruta al directorio de assets
pub fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Project root not found")
        .join("assets")
}

/// Verificar si un archivo existe
pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Inicializar logging (placeholder para tracing)
pub fn init_logging() {
    #[cfg(debug_assertions)]
    {
        // En desarrollo, los logs van a stdout
    }
    #[cfg(not(debug_assertions))]
    {
        // En producción, configurar tracing-appender si es necesario
    }
}
