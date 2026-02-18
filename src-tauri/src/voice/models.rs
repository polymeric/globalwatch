use std::path::PathBuf;
use tauri::Manager;

/// Returns the directory where voice models are stored.
/// Uses Tauri's app_data_dir / "models" subdirectory.
pub fn models_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let dir = base.join("models");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create models dir: {e}"))?;
    Ok(dir)
}

/// Returns expected path for the whisper model file.
pub fn whisper_model_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(models_dir(app)?.join("ggml-base.en.bin"))
}

/// Check whether whisper model exists on disk.
pub fn whisper_model_exists(app: &tauri::AppHandle) -> Result<bool, String> {
    Ok(whisper_model_path(app)?.exists())
}
