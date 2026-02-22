use std::process::Command;

/// Speak text using the platform's available TTS command.
/// Blocks until speech finishes.
pub fn speak(text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return speak_macos(text);

    #[cfg(target_os = "linux")]
    return speak_linux(text);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    Err("TTS not supported on this platform".to_string())
}

// ── macOS ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
const PREFERRED_VOICES: &[&str] = &[
    "Zoe (Enhanced)",
    "Joelle (Enhanced)",
    "Samantha",
    "Daniel",
    "Karen",
];

#[cfg(target_os = "macos")]
fn speak_macos(text: &str) -> Result<(), String> {
    let voice = pick_macos_voice();
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

#[cfg(target_os = "macos")]
fn pick_macos_voice() -> String {
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

// ── Linux ────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn speak_linux(text: &str) -> Result<(), String> {
    // cpal microphone sessions switch BT headsets (e.g. AirPods) from A2DP to
    // headset-head-unit (HFP). Audio output through HFP is unreliable on Linux.
    // Switch any such cards back to a2dp-sink before playing.
    let switched = restore_bt_a2dp();
    if switched > 0 {
        // Give BlueZ time to re-establish the A2DP connection.
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    let espeak = if cmd_exists("espeak-ng") {
        Some("espeak-ng")
    } else if cmd_exists("espeak") {
        Some("espeak")
    } else {
        None
    };

    if let Some(espeak) = espeak {
        let wav = std::env::temp_dir().join("globalwatch_tts.wav");
        run(Command::new(espeak).arg("-w").arg(&wav).arg(text))?;

        if cmd_exists("paplay") {
            run(Command::new("paplay").arg(&wav))?;
            log::info!("[tts] spoke via {espeak} + paplay");
        } else {
            run(Command::new(espeak).arg(text))?;
            log::info!("[tts] spoke via {espeak}");
        }
        return Ok(());
    }

    if cmd_exists("spd-say") {
        run(Command::new("spd-say").arg("--wait").arg(text))?;
        log::info!("[tts] spoke via spd-say");
        return Ok(());
    }

    Err("No TTS engine found. Install one with: sudo pacman -S espeak-ng".to_string())
}

/// Find Bluetooth cards whose active profile is a headset/HFP variant and
/// switch them to a2dp-sink so audio output works during TTS playback.
/// Returns the number of cards switched.
#[cfg(target_os = "linux")]
fn restore_bt_a2dp() -> u32 {
    let output = match Command::new("pactl").args(["list", "cards"]).output() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    let text = String::from_utf8_lossy(&output.stdout);

    let mut current_card = String::new();
    let mut switched = 0u32;

    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("Name: ") {
            current_card = name.to_string();
        } else if line.starts_with("Active Profile: headset") && current_card.contains("bluez_card") {
            match Command::new("pactl")
                .args(["set-card-profile", &current_card, "a2dp-sink"])
                .status()
            {
                Ok(s) if s.success() => {
                    log::info!("[tts] switched {} from HFP to a2dp-sink", current_card);
                    switched += 1;
                }
                Ok(s) => log::warn!("[tts] set-card-profile failed for {}: {}", current_card, s),
                Err(e) => log::warn!("[tts] set-card-profile error for {}: {}", current_card, e),
            }
        }
    }
    switched
}

#[cfg(target_os = "linux")]
fn cmd_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run `{:?}`: {e}", cmd.get_program()))?;
    if !status.success() {
        return Err(format!("`{:?}` exited with status: {status}", cmd.get_program()));
    }
    Ok(())
}
