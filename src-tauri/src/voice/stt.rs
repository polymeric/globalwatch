use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::path::Path;

pub struct WhisperState {
    ctx: Option<WhisperContext>,
}

impl WhisperState {
    pub fn new() -> Self {
        Self { ctx: None }
    }

    /// Load the whisper model from disk.
    pub fn load_model(&mut self, model_path: &Path) -> Result<(), String> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or("Invalid model path")?,
            params,
        )
        .map_err(|e| format!("Failed to load whisper model: {e}"))?;
        self.ctx = Some(ctx);
        Ok(())
    }

    /// Transcribe f32 PCM audio at 16kHz mono.
    pub fn transcribe(&self, audio_f32_16khz: &[f32]) -> Result<String, String> {
        let ctx = self.ctx.as_ref().ok_or("Whisper model not loaded")?;

        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Failed to create whisper state: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(false);
        params.set_no_context(true);
        params.set_suppress_blank(true);

        state
            .full(params, audio_f32_16khz)
            .map_err(|e| format!("Whisper transcription failed: {e}"))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| format!("Failed to get segments: {e}"))?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }
        Ok(text.trim().to_string())
    }
}
