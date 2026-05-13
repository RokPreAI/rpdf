use crate::contracts::dto::SpeakTextRequestDto;
use std::process::Command;

pub fn local_tts_available() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v spd-say >/dev/null 2>&1")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn speak_text(request: &SpeakTextRequestDto) -> Result<(), String> {
    let normalized_text = request.text.trim();

    if normalized_text.is_empty() {
        return Err("Text is required for local speech.".to_string());
    }

    if !local_tts_available() {
        return Err("Local speech backend is unavailable on this machine.".to_string());
    }

    let rate = speech_rate_to_spd_rate(request.rate);
    let output = Command::new("spd-say")
        .args([
            "--wait",
            "--application-name",
            "rpdf",
            "--rate",
            &rate.to_string(),
            normalized_text,
        ])
        .output()
        .map_err(|error| format!("Could not run spd-say: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("spd-say failed: {}", stderr.trim()))
}

pub fn stop_speaking() -> Result<(), String> {
    if !local_tts_available() {
        return Ok(());
    }

    let output = Command::new("spd-say")
        .arg("--stop")
        .output()
        .map_err(|error| format!("Could not stop spd-say: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("spd-say --stop failed: {}", stderr.trim()))
}

fn speech_rate_to_spd_rate(rate: f32) -> i32 {
    let normalized_rate = rate.clamp(0.5, 2.0);
    ((normalized_rate - 1.0) * 100.0).round() as i32
}
