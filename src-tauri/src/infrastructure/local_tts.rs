use crate::contracts::dto::SpeakTextRequestDto;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

    if let Some(diagnosis) = infer_speech_dispatcher_diagnosis(trimmed) {
        return format!("{prefix}: {diagnosis}");
    }

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

fn infer_speech_dispatcher_diagnosis(stderr_text: &str) -> Option<String> {
    if let Some(stderr_diagnosis) = diagnose_from_text_sources([stderr_text]) {
        return Some(stderr_diagnosis);
    }

    if should_consult_runtime_logs(stderr_text) {
        return diagnose_from_runtime_logs();
    }

    None
}

fn should_consult_runtime_logs(stderr_text: &str) -> bool {
    let normalized = stderr_text.trim();

    normalized.is_empty()
        || normalized.contains("Speech Dispatcher")
        || normalized.contains("speechd")
        || normalized.contains("output module")
        || normalized.contains("MODULE ERROR")
        || normalized.contains("audio")
        || normalized.contains("voice")
}

fn diagnose_from_runtime_logs() -> Option<String> {
    let mut sources = Vec::new();

    for log_path in candidate_speech_dispatcher_log_paths() {
        if let Ok(contents) = fs::read_to_string(&log_path) {
            if !contents.trim().is_empty() {
                sources.push(contents);
            }
        }
    }

    if sources.is_empty() {
        return None;
    }

    diagnose_from_text_sources(sources.iter().map(String::as_str))
}

fn candidate_speech_dispatcher_log_paths() -> Vec<PathBuf> {
    let Some(log_dir) = speech_dispatcher_log_dir() else {
        return Vec::new();
    };

    [
        "speech-dispatcher.log",
        "espeak-ng.log",
        "espeak-ng-fallback.log",
        "espeak-ng-mbrola.log",
        "festival.log",
        "openjtalk.log",
        "dummy.log",
    ]
    .into_iter()
    .map(|name| log_dir.join(name))
    .collect()
}

fn speech_dispatcher_log_dir() -> Option<PathBuf> {
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        let candidate = Path::new(&runtime_dir).join("speech-dispatcher/log");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if let Ok(uid) = env::var("UID") {
        let candidate = PathBuf::from(format!("/run/user/{uid}/speech-dispatcher/log"));
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn diagnose_from_text_sources<'a, I>(sources: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let collected: Vec<&str> = sources.into_iter().collect();

    if collected.is_empty() {
        return None;
    }

    let has_missing_espeak_lib = collected
        .iter()
        .any(|source| source.contains("libespeak-ng.so.1"));
    let has_dummy_audio_failure = collected
        .iter()
        .any(|source| source.contains("Opening sound device failed. Reason: server audio is not supported."));
    let has_festival_connection_failure = collected
        .iter()
        .any(|source| source.contains("festival_client: connect to server failed"));
    let has_openjtalk_voice_failure = collected.iter().any(|source| {
        source.contains("nitech_jp_atr503_m001.htsvoice") || source.contains("open: No such file or directory")
    });

    if has_missing_espeak_lib && has_dummy_audio_failure {
        return Some("Speech Dispatcher is running, but it has no usable speech backend: the espeak-ng modules cannot load because `libespeak-ng.so.1` is missing, and the remaining dummy backend cannot open audio. Install the package that provides `libespeak-ng.so.1` or configure another real Speech Dispatcher output module.".to_string());
    }

    if has_missing_espeak_lib {
        return Some(
            "Speech Dispatcher is running, but its espeak-ng output modules cannot load because `libespeak-ng.so.1` is missing. Install the package that provides `libespeak-ng.so.1` or configure another real Speech Dispatcher output module.".to_string(),
        );
    }

    if has_dummy_audio_failure {
        return Some(
            "Speech Dispatcher is running, but the remaining backend cannot open audio (`server audio is not supported`). Configure a working audio-capable Speech Dispatcher output module.".to_string(),
        );
    }

    if has_festival_connection_failure {
        return Some(
            "Speech Dispatcher tried the Festival backend, but no Festival server is running. Start Festival or configure a different Speech Dispatcher output module.".to_string(),
        );
    }

    if has_openjtalk_voice_failure {
        return Some(
            "Speech Dispatcher tried the OpenJTalk backend, but its voice data is missing. Install the required OpenJTalk voice package or configure a different Speech Dispatcher output module.".to_string(),
        );
    }

    diagnose_from_stderr_only(&collected)
}

fn diagnose_from_stderr_only(lines: &[&str]) -> Option<String> {
    for entry in lines {
        for line in entry.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if line.contains("speechd.sock") {
                return Some(line.to_string());
            }
        }
    }

    for entry in lines {
        for line in entry.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if line.contains("Failed to connect to Speech Dispatcher") {
                return Some("could not connect to Speech Dispatcher.".to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{diagnose_from_text_sources, format_speech_dispatcher_error};

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

    #[test]
    fn diagnoses_missing_espeak_and_dummy_audio_failure() {
        let diagnosis = diagnose_from_text_sources([
            "/usr/lib/speech-dispatcher/speech-dispatcher-modules/sd_espeak-ng: error while loading shared libraries: libespeak-ng.so.1: cannot open shared object file: No such file or directory",
            "300-Opening sound device failed. Reason: server audio is not supported.",
        ]);

        assert_eq!(
            diagnosis,
            Some("Speech Dispatcher is running, but it has no usable speech backend: the espeak-ng modules cannot load because `libespeak-ng.so.1` is missing, and the remaining dummy backend cannot open audio. Install the package that provides `libespeak-ng.so.1` or configure another real Speech Dispatcher output module.".to_string())
        );
    }

    #[test]
    fn diagnoses_festival_backend_failure() {
        let diagnosis =
            diagnose_from_text_sources(["festival_client: connect to server failed"]);

        assert_eq!(
            diagnosis,
            Some("Speech Dispatcher tried the Festival backend, but no Festival server is running. Start Festival or configure a different Speech Dispatcher output module.".to_string())
        );
    }
}
