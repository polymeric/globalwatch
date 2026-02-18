use std::process::Command;

/// Preferred voices in order — first available one is used.
const PREFERRED_VOICES: &[&str] = &[
    "Zoe (Enhanced)",
    "Joelle (Enhanced)",
    "Samantha",
    "Daniel",
    "Karen",
];

/// Speak text using macOS `say` command with a natural-sounding voice.
/// Blocks until speech finishes.
pub fn speak(text: &str) -> Result<(), String> {
    let voice = pick_voice();
    log::info!("[tts] speaking with voice: {}", voice);

    let status = Command::new("say")
        .arg("-v")
        .arg(&voice)
        .arg(text)
        .status()
        .map_err(|e| format!("Failed to run `say`: {e}"))?;

    if !status.success() {
        return Err(format!("`say` exited with status: {status}"));
    }
    Ok(())
}

/// Pick the best available voice by checking installed voices.
fn pick_voice() -> String {
    if let Ok(output) = Command::new("say").arg("-v").arg("?").output() {
        let list = String::from_utf8_lossy(&output.stdout);
        for &name in PREFERRED_VOICES {
            if list.contains(name) {
                return name.to_string();
            }
        }
    }
    "Samantha".to_string()
}
