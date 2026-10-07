//! "Start with Windows", for both ways Keyflex can be installed.
//!
//! A normal install adds an entry under the registry's Run key (the autostart
//! plugin). A Microsoft Store install cannot: its registry writes are private
//! to the package, so Windows would never see them. There the package declares
//! a startup task (see `packaging/AppxManifest.xml`) and this module switches
//! it on and off.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use windows::core::HSTRING;
use windows::ApplicationModel::Activation::ActivationKind;
use windows::ApplicationModel::{AppInstance, StartupTask, StartupTaskState};
use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

/// Passed by the Run-key entry, so signing in does not open the window.
pub const AUTOSTART_ARG: &str = "--autostart";
/// Must match the TaskId in `packaging/AppxManifest.xml`.
const STARTUP_TASK_ID: &str = "KeyflexStartup";

/// Whether this copy was installed as a package (Microsoft Store / MSIX).
pub fn is_packaged() -> bool {
    let mut length = 0u32;
    unsafe { GetCurrentPackageFullName(&mut length, windows::core::PWSTR::null()) != APPMODEL_ERROR_NO_PACKAGE }
}

/// Whether Windows started us at sign-in, as opposed to the user opening the app.
pub fn launched_at_sign_in() -> bool {
    if std::env::args().any(|arg| arg == AUTOSTART_ARG) {
        return true;
    }
    is_packaged()
        && AppInstance::GetActivatedEventArgs()
            .and_then(|args| args.Kind())
            .is_ok_and(|kind| kind == ActivationKind::StartupTask)
}

fn startup_task() -> windows::core::Result<StartupTask> {
    StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))?.get()
}

/// Blocks briefly when packaged: call from a background thread.
pub fn is_enabled(app: &AppHandle) -> bool {
    if !is_packaged() {
        return app.autolaunch().is_enabled().unwrap_or(false);
    }
    startup_task()
        .and_then(|task| task.State())
        .is_ok_and(|state| state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy)
}

/// Blocks briefly when packaged: call from a background thread.
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    if !is_packaged() {
        let autolaunch = app.autolaunch();
        return if enabled { autolaunch.enable() } else { autolaunch.disable() }.map_err(|e| e.to_string());
    }

    let task = startup_task().map_err(|e| e.message())?;
    if !enabled {
        return task.Disable().map_err(|e| e.message());
    }
    let state = task.RequestEnableAsync().and_then(|request| request.get()).map_err(|e| e.message())?;
    match state {
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy => Ok(()),
        // Once the user has switched it off in Windows, only they can switch it back on.
        StartupTaskState::DisabledByUser => {
            Err("Windows has this switched off. Turn Keyflex on in Settings > Apps > Startup.".into())
        }
        _ => Err("Your organisation's settings do not allow this.".into()),
    }
}
