#[cfg(target_os = "linux")]
use gdk::{AxisUse, EventButton, EventMask, EventMotion, InputSource};
#[cfg(target_os = "linux")]
use glib::Propagation;
#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use serde::Serialize;
#[cfg(target_os = "linux")]
use tauri::{AppHandle, Emitter, Manager, Runtime, Webview};

#[cfg(target_os = "linux")]
const NATIVE_PRESSURE_EVENT: &str = "rpdf://native-pressure";

#[cfg(target_os = "linux")]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativePressureSamplePayload {
    pressure: f64,
    source: String,
    device_name: Option<String>,
    updated_at_ms: u64,
    is_stylus_like: bool,
}

#[cfg(target_os = "linux")]
pub fn install<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let webview = app
        .get_webview_window("main")
        .ok_or_else(|| "Could not find main webview window for native Linux pressure bridge.".to_string())?;

    attach_to_webview(app, webview.as_ref())
}

#[cfg(not(target_os = "linux"))]
pub fn install<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn attach_to_webview<R: Runtime>(app: &AppHandle<R>, webview: &Webview<R>) -> Result<(), String> {
    let app_handle = app.clone();

    webview
        .with_webview(move |platform_webview| {
            let webview = platform_webview.inner();
            webview.add_events(
                EventMask::BUTTON_PRESS_MASK
                    | EventMask::BUTTON_RELEASE_MASK
                    | EventMask::POINTER_MOTION_MASK
                    | EventMask::LEAVE_NOTIFY_MASK
                    | EventMask::PROXIMITY_IN_MASK
                    | EventMask::PROXIMITY_OUT_MASK,
            );

            let motion_app = app_handle.clone();
            webview.connect_motion_notify_event(move |_, event| {
                emit_event_pressure(&motion_app, event);
                Propagation::Proceed
            });

            let press_app = app_handle.clone();
            webview.connect_button_press_event(move |_, event| {
                emit_button_pressure(&press_app, event);
                Propagation::Proceed
            });

            let release_app = app_handle.clone();
            webview.connect_button_release_event(move |_, event| {
                emit_button_pressure(&release_app, event);
                Propagation::Proceed
            });

            let leave_app = app_handle.clone();
            webview.connect_leave_notify_event(move |_, _| {
                emit_payload(
                    &leave_app,
                    NativePressureSamplePayload {
                        pressure: 0.0,
                        source: "Leave".to_string(),
                        device_name: None,
                        updated_at_ms: now_ms(),
                        is_stylus_like: false,
                    },
                );
                Propagation::Proceed
            });
        })
        .map_err(|error| format!("Could not attach Linux pressure bridge to webview: {error}"))
}

#[cfg(target_os = "linux")]
fn emit_event_pressure<R: Runtime>(app: &AppHandle<R>, event: &EventMotion) {
    let payload = event
        .axis(AxisUse::Pressure)
        .map(|pressure| pressure_payload(event.source_device().or_else(|| event.device()), pressure));

    if let Some(payload) = payload {
        emit_payload(app, payload);
    }
}

#[cfg(target_os = "linux")]
fn emit_button_pressure<R: Runtime>(app: &AppHandle<R>, event: &EventButton) {
    let pressure = event.axis(AxisUse::Pressure).unwrap_or(0.0);
    let payload = pressure_payload(event.source_device().or_else(|| event.device()), pressure);
    emit_payload(app, payload);
}

#[cfg(target_os = "linux")]
fn pressure_payload(device: Option<gdk::Device>, pressure: f64) -> NativePressureSamplePayload {
    let normalized_pressure = pressure.clamp(0.0, 1.0);
    let source = device
        .as_ref()
        .map(device_source_label)
        .unwrap_or_else(|| "Unknown".to_string());
    let is_stylus_like = device
        .as_ref()
        .map(device_is_stylus_like)
        .unwrap_or(false);
    let device_name = device
        .as_ref()
        .and_then(|value| value.name())
        .map(|value| value.to_string());

    NativePressureSamplePayload {
        pressure: normalized_pressure,
        source,
        device_name,
        updated_at_ms: now_ms(),
        is_stylus_like,
    }
}

#[cfg(target_os = "linux")]
fn emit_payload<R: Runtime>(app: &AppHandle<R>, payload: NativePressureSamplePayload) {
    let _ = app.emit_to("main", NATIVE_PRESSURE_EVENT, payload);
}

#[cfg(target_os = "linux")]
fn device_source_label(device: &gdk::Device) -> String {
    match device.source() {
        InputSource::Mouse => "Mouse".to_string(),
        InputSource::Pen => "Pen".to_string(),
        InputSource::Eraser => "Eraser".to_string(),
        InputSource::Cursor => "Cursor".to_string(),
        InputSource::Keyboard => "Keyboard".to_string(),
        InputSource::Touchscreen => "Touchscreen".to_string(),
        InputSource::Touchpad => "Touchpad".to_string(),
        InputSource::Trackpoint => "Trackpoint".to_string(),
        InputSource::TabletPad => "TabletPad".to_string(),
        InputSource::__Unknown(value) => format!("Unknown({value})"),
        _ => "Unknown".to_string(),
    }
}

#[cfg(target_os = "linux")]
fn device_is_stylus_like(device: &gdk::Device) -> bool {
    matches!(device.source(), InputSource::Pen | InputSource::Eraser | InputSource::Cursor)
}

#[cfg(target_os = "linux")]
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
