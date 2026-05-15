use crate::contracts::dto::SpeakTextRequestDto;
use std::process::Command;

pub fn speak_text(request: &SpeakTextRequestDto) -> Result<(), String> {
    let normalized_text = request.text.trim();

    if normalized_text.is_empty() {
        return Err("Text is required for local speech.".to_string());
    }

    local_tts_status()?;

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

    Err(format_speech_dispatcher_error(
        "Local speech playback failed",
        &output.stderr,
    ))
}

pub fn stop_speaking() -> Result<(), String> {
    if command_exists("spd-say").is_err() {
        return Ok(());
    }

    let output = Command::new("spd-say")
        .arg("--stop")
        .output()
        .map_err(|error| format!("Could not stop spd-say: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format_speech_dispatcher_error(
        "Could not stop local speech",
        &output.stderr,
    ))
}

fn speech_rate_to_spd_rate(rate: f32) -> i32 {
    let normalized_rate = rate.clamp(0.5, 2.0);
    ((normalized_rate - 1.0) * 100.0).round() as i32
}

fn local_tts_status() -> Result<(), String> {
    command_exists("spd-say")?;

    let output = Command::new("spd-say")
        .arg("-O")
        .output()
        .map_err(|error| format!("Could not check local speech backend: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format_speech_dispatcher_error(
        "Local speech backend is installed but unavailable",
        &output.stderr,
    ))
}

fn command_exists(command: &str) -> Result<(), String> {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map_err(|error| format!("Could not check for {command}: {error}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "Local speech backend is unavailable because `{command}` is not installed."
                ))
            }
        })
}

fn format_speech_dispatcher_error(prefix: &str, stderr: &[u8]) -> String {
    let stderr_text = String::from_utf8_lossy(stderr);
    let trimmed = stderr_text.trim();

    if trimmed.is_empty() {
        return prefix.to_string();
    }

    let mut lines = trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let first = lines.next().unwrap_or(prefix);
    let second = lines.next();

    if first.contains("Failed to connect to Speech Dispatcher") {
        if let Some(detail) = second {
            return format!("{prefix}: {detail}");
        }

        return format!("{prefix}: could not connect to Speech Dispatcher.");
    }

    format!("{prefix}: {first}")
}

#[cfg(test)]
mod tests {
    use super::format_speech_dispatcher_error;

    #[test]
    fn formats_dispatcher_connection_errors_compactly() {
        let stderr = b"Failed to connect to Speech Dispatcher:\nError: Can't connect to unix socket /run/user/1000/speech-dispatcher/speechd.sock: Operation not permitted.\n";
        let message = format_speech_dispatcher_error("Local speech backend is installed but unavailable", stderr);

        assert_eq!(
            message,
            "Local speech backend is installed but unavailable: Error: Can't connect to unix socket /run/user/1000/speech-dispatcher/speechd.sock: Operation not permitted."
        );
    }

    #[test]
    fn formats_generic_errors_compactly() {
        let stderr = b"some generic failure\nextra detail we do not want to surface\n";
        let message = format_speech_dispatcher_error("Local speech playback failed", stderr);

        assert_eq!(message, "Local speech playback failed: some generic failure");
    }
}
