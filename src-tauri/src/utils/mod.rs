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
