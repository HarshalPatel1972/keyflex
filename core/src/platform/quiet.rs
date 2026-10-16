//! Moments when nothing should pop up: a fullscreen app or slideshow, a
//! projector showing a copy of the screen, or a call, where screens get shared.
//! Asked only when something is about to pop up, never on a timer.

use windows::core::Interface;
use windows::Win32::Devices::Display::*;
use windows::Win32::Foundation::*;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;

use crate::platform::watch;

/// Why now is a bad moment to pop up, or `None` if it is fine.
pub fn reason() -> Option<String> {
    unsafe {
        if let Some(reason) = busy_screen() {
            return Some(reason.into());
        }
        if mirrored() {
            return Some("the screen is mirrored to another display".into());
        }
        // Already set up on the worker thread; this only matters elsewhere.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        recording()
    }
}

/// Windows' own verdict, the one it uses to hold back its notifications.
unsafe fn busy_screen() -> Option<&'static str> {
    match SHQueryUserNotificationState().ok()? {
        // Covers slideshows too: PowerPoint's fills the screen.
        QUNS_BUSY | QUNS_APP => Some("a fullscreen app is in front"),
        QUNS_RUNNING_D3D_FULL_SCREEN => Some("a fullscreen game or video is in front"),
        QUNS_PRESENTATION_MODE => Some("presentation mode is on"),
        QUNS_NOT_PRESENT => Some("the screen is locked"),
        _ => None,
    }
}

/// The displays show one picture between them: a laptop on a projector or TV.
unsafe fn mirrored() -> bool {
    let (mut paths, mut modes) = (0u32, 0u32);
    if GetDisplayConfigBufferSizes(QDC_DATABASE_CURRENT, &mut paths, &mut modes) != ERROR_SUCCESS {
        return false;
    }
    let mut path_info = vec![DISPLAYCONFIG_PATH_INFO::default(); paths as usize];
    let mut mode_info = vec![DISPLAYCONFIG_MODE_INFO::default(); modes as usize];
    let mut topology = DISPLAYCONFIG_TOPOLOGY_ID::default();
    let read = QueryDisplayConfig(
        QDC_DATABASE_CURRENT,
        &mut paths,
        path_info.as_mut_ptr(),
        &mut modes,
        mode_info.as_mut_ptr(),
        Some(&mut topology),
    );
    read == ERROR_SUCCESS && topology == DISPLAYCONFIG_TOPOLOGY_CLONE
}

/// An app is recording from a microphone: a call, most likely, and calls are
/// where screens get shared. Windows has no way to ask about the sharing itself.
unsafe fn recording() -> Option<String> {
    let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
    let microphones = devices.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE).ok()?;
    for i in 0..microphones.GetCount().ok()? {
        let Ok(manager) = microphones.Item(i).and_then(|mic| mic.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None))
        else {
            continue;
        };
        let Ok(sessions) = manager.GetSessionEnumerator() else { continue };
        for j in 0..sessions.GetCount().unwrap_or(0) {
            let Ok(session) = sessions.GetSession(j) else { continue };
            if session.GetState() == Ok(AudioSessionStateActive) {
                let app = session
                    .cast::<IAudioSessionControl2>()
                    .and_then(|session| session.GetProcessId())
                    .map(watch::app_of_process)
                    .unwrap_or_default();
                return Some(format!("{} is using the microphone", if app.is_empty() { "an app" } else { &app }));
            }
        }
    }
    None
}
