# Title

Linux native pressure bridge report

# Context

- Problem:
  - The canvas pressure toggle and width model were already implemented, but live Linux testing on a Wacom Intuos still showed `Pressure: 1.000` all the time in the Tauri app.
  - Prior debugging and research pointed at WebKitGTK pointer semantics rather than stroke math: the frontend path was likely receiving stylus input as mouse-like events, so browser `PointerEvent.pressure` could not be trusted on this stack.
- Constraints:
  - This needed to stay a bounded pressure-only slice.
  - The repo still had unrelated staged deletions and generated build churn, so the commit had to stay path-limited.

# Goals

- Primary success criteria:
  - add a Linux-native pressure source on the Rust side
  - forward normalized pressure samples into the existing canvas input pipeline
  - keep browser pressure as fallback where it still works
  - expose enough debug information to confirm whether the frontend event is still arriving as mouse-like input
- Secondary success criteria:
  - avoid redesigning the renderer or canvas model
  - keep the change Linux-specific instead of destabilizing other platforms

# Approach

- Chosen approach:
  - attach GTK event listeners to the live Tauri webview on Linux
  - read `GDK_AXIS_PRESSURE` from native motion and button events
  - emit compact pressure payloads back to the `main` webview through a Tauri event
  - let the canvas keep using browser pointer coordinates while swapping in native pressure when the browser event looks mouse-like
- Rejected options:
  - polling a backend command on every pointer move would have been much heavier than the event bridge
  - continuing to tune browser pressure math would not solve the underlying Linux/WebKitGTK input limitation

# Implementation

- Task hash:
  - `add-linux-native-pressure-bridge`
- Matching task file:
  - [todos/add-linux-native-pressure-bridge](/home/rok/sync/ideas/rpdf2/todos/add-linux-native-pressure-bridge:1)
- Architecture / flow:
  - On Linux startup, the Tauri backend now attaches to the `main` webview window and hooks GTK motion, press, release, and leave events.
  - Native pressure samples are read from GDK axis data, normalized to `0..1`, tagged with device source metadata, and emitted to the frontend as `rpdf://native-pressure`.
  - Canvas Mode listens for that event and keeps the freshest recent native pressure sample.
  - When a browser pointer event still reports `pointerType === "mouse"`, the canvas now prefers the native Linux pressure sample instead of hard-forcing `1`.
  - The temporary pressure debug line now shows the effective pressure, the browser pointer type/raw pressure, and the latest native source/pressure so the Linux input shape is visible during testing.
- Key files or components:
  - [src-tauri/src/infrastructure/linux_pressure.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/infrastructure/linux_pressure.rs:1)
    - new Linux-only GTK/GDK pressure bridge
    - emits `rpdf://native-pressure` samples to the `main` webview
  - [src-tauri/src/lib.rs](/home/rok/sync/ideas/rpdf2/src-tauri/src/lib.rs:1)
    - installs the native pressure bridge during Tauri setup
  - [src-tauri/Cargo.toml](/home/rok/sync/ideas/rpdf2/src-tauri/Cargo.toml:1)
    - adds Linux-targeted `gdk`, `glib`, `gtk`, and `webkit2gtk` dependencies for the bridge
  - [src/features/canvas/workspace.ts](/home/rok/sync/ideas/rpdf2/src/features/canvas/workspace.ts:1)
    - listens for native pressure events
    - stores the freshest recent Linux pressure sample
    - prefers that sample when browser input looks mouse-like
    - expands the pressure debug readout and pointerdown logging
  - [TODO.md](/home/rok/sync/ideas/rpdf2/TODO.md:1)
    - marks `add-linux-native-pressure-bridge` done
  - [todos/add-linux-native-pressure-bridge](/home/rok/sync/ideas/rpdf2/todos/add-linux-native-pressure-bridge:1)
    - marks the task done

# Results

- Outputs:
  - The app now has a real Linux-native pressure path independent of browser `PointerEvent.pressure`.
  - Canvas drawing can use GTK/GDK pressure samples when WebKitGTK downgrades stylus input to mouse-like browser events.
  - The debug line now makes the mismatch visible in-app instead of hiding it behind a single constant value.
- Verification:
  - Ran `npm run build`
    - Result: success
  - Ran `cargo check --manifest-path src-tauri/Cargo.toml`
    - Result: success

# Decisions

- Fact:
  - The bridge emits pressure samples continuously and the frontend still uses the browser event stream for coordinates, selection, and timing.
- Assessment:
  - That is the smallest change that fixes pressure without forking the whole input stack into native code.

# Limitations

- I did not manually verify the live Wacom Intuos behavior in the running Tauri app in this turn, so the real remaining question is whether the GTK pressure events line up well enough with the browser pointer stream under your desktop/session.
- The bridge currently targets Linux only and does not attempt tilt, rotation, eraser-specific behavior changes, or pad buttons.
- The debug readout is still temporary and more verbose than the final UI should be.

# Next steps

1. Run the app on the Wacom setup and confirm the toolbar now shows changing native pressure instead of a fixed `1.000`.
2. Check the pointerdown console log once to confirm whether the browser event is still arriving as `pointerType: "mouse"`.
3. If the live pressure now works, add a small cleanup task to remove or shrink the debug instrumentation.
4. If pressure still does not vary, capture the new debug line plus the pointerdown log and add a narrower follow-up task around GTK event coverage or event freshness alignment.

# Reproducibility

1. Build the frontend:
   - `npm run build`
2. Check the Rust/Tauri backend:
   - `cargo check --manifest-path src-tauri/Cargo.toml`
3. Launch the app on Linux, switch to Canvas Mode, enable `Pressure`, draw with the tablet, and watch the toolbar pressure debug line while checking the console for the `pointerdown` input diagnostic.
