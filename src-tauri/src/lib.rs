mod voice;

use std::sync::Mutex;
use voice::capture::AudioCapture;
use voice::stt::WhisperState;

#[tauri::command]
async fn check_models_ready(app: tauri::AppHandle) -> Result<bool, String> {
    voice::models::whisper_model_exists(&app)
}

#[tauri::command]
async fn get_models_dir(app: tauri::AppHandle) -> Result<String, String> {
    voice::models::models_dir(&app).map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
async fn load_whisper_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<WhisperState>>,
) -> Result<(), String> {
    let model_path = voice::models::whisper_model_path(&app)?;
    if !model_path.exists() {
        return Err(format!(
            "Whisper model not found. Place ggml-base.en.bin in: {}",
            model_path.parent().unwrap_or(&model_path).display()
        ));
    }
    log::info!("Loading whisper model from: {}", model_path.display());
    let mut whisper = state.lock().map_err(|e| format!("Lock error: {e}"))?;
    whisper.load_model(&model_path)?;
    log::info!("Whisper model loaded successfully");
    Ok(())
}

/// Start recording audio from the default input device.
#[tauri::command]
async fn start_recording(
    capture: tauri::State<'_, Mutex<AudioCapture>>,
) -> Result<(), String> {
    log::info!("start_recording called");
    let cap = capture.lock().map_err(|e| format!("Lock error: {e}"))?;
    cap.start()
}

/// Stop recording and transcribe the captured audio with whisper.
/// Audio stays entirely on the Rust side — no IPC transfer of raw audio bytes.
#[tauri::command]
async fn stop_and_transcribe(
    capture: tauri::State<'_, Mutex<AudioCapture>>,
    whisper: tauri::State<'_, Mutex<WhisperState>>,
) -> Result<String, String> {
    log::info!("stop_and_transcribe called");

    let samples = {
        let cap = capture.lock().map_err(|e| format!("Lock error: {e}"))?;
        cap.stop()?
    };

    if samples.len() < 1600 {
        log::warn!("Audio too short (<0.1s at 16kHz), skipping transcription");
        return Ok(String::new());
    }

    log::info!(
        "Transcribing {} samples ({:.1}s of audio)",
        samples.len(),
        samples.len() as f32 / 16000.0
    );

    let w = whisper.lock().map_err(|e| format!("Lock error: {e}"))?;
    let text = w.transcribe(&samples)?;
    log::info!("Whisper result: {:?}", text);
    Ok(text)
}

#[tauri::command]
async fn query_llm(
    prompt: String,
    history: Vec<voice::llm::ChatMessage>,
) -> Result<String, String> {
    log::info!("query_llm: prompt={:?}, history_len={}", prompt, history.len());
    let mut messages = history;
    messages.push(voice::llm::ChatMessage {
        role: "user".to_string(),
        content: prompt,
    });
    let result = voice::llm::query_lm_studio(messages).await;
    match &result {
        Ok(resp) => log::info!("LLM response: {:?}", resp),
        Err(e) => log::error!("LLM error: {}", e),
    }
    result
}

#[tauri::command]
async fn speak_text(text: String) -> Result<(), String> {
    log::info!("speak_text: {:?}", text);
    // Run on a blocking thread so we don't tie up the async runtime
    tokio::task::spawn_blocking(move || voice::tts::speak(&text))
        .await
        .map_err(|e| format!("TTS task failed: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(WhisperState::new()))
        .manage(Mutex::new(AudioCapture::new()))
        .setup(|app| {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_models_ready,
            get_models_dir,
            load_whisper_model,
            start_recording,
            stop_and_transcribe,
            query_llm,
            speak_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
