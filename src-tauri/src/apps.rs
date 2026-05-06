#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::System::Threading::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

/// Static lookup map: exe filename → friendly display name
static APP_NAME_MAP: &[(&str, &str)] = &[
    ("Code.exe", "VS Code"),
    ("Code - Insiders.exe", "VS Code Insiders"),
    ("chrome.exe", "Chrome"),
    ("firefox.exe", "Firefox"),
    ("msedge.exe", "Edge"),
    ("brave.exe", "Brave"),
    ("opera.exe", "Opera"),
    ("notepad.exe", "Notepad"),
    ("notepad++.exe", "Notepad++"),
    ("WindowsTerminal.exe", "Windows Terminal"),
    ("wt.exe", "Windows Terminal"),
    ("powershell.exe", "PowerShell"),
    ("cmd.exe", "Command Prompt"),
    ("explorer.exe", "File Explorer"),
    ("slack.exe", "Slack"),
    ("discord.exe", "Discord"),
    ("Teams.exe", "Microsoft Teams"),
    ("OUTLOOK.EXE", "Outlook"),
    ("WINWORD.EXE", "Word"),
    ("EXCEL.EXE", "Excel"),
    ("POWERPNT.EXE", "PowerPoint"),
    ("figma.exe", "Figma"),
    ("rider64.exe", "Rider"),
    ("idea64.exe", "IntelliJ IDEA"),
    ("webstorm64.exe", "WebStorm"),
    ("studio64.exe", "Android Studio"),
    ("cursor.exe", "Cursor"),
    ("windsurf.exe", "Windsurf"),
    ("sublime_text.exe", "Sublime Text"),
    ("atom.exe", "Atom"),
    ("spotify.exe", "Spotify"),
    ("vlc.exe", "VLC"),
    ("obs64.exe", "OBS Studio"),
    ("postman.exe", "Postman"),
    ("insomnia.exe", "Insomnia"),
    ("TablePlus.exe", "TablePlus"),
    ("GitKraken.exe", "GitKraken"),
    ("SourceTree.exe", "SourceTree"),
];

/// Get the exe name and friendly name of the currently focused window.
/// Returns (friendly_name, exe_path).
#[cfg(windows)]
pub fn get_foreground_app() -> (String, String) {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return ("Unknown".to_string(), String::new());
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return ("Unknown".to_string(), String::new());
        }

        let process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(p) => p,
            Err(_) => return ("Unknown".to_string(), String::new()),
        };

        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = windows::Win32::Foundation::CloseHandle(process);

        if result.is_err() {
            return ("Unknown".to_string(), String::new());
        }

        let exe_path = String::from_utf16_lossy(&buf[..size as usize]);
        let exe_name = exe_path
            .rsplit('\\')
            .next()
            .unwrap_or("")
            .to_string();

        let friendly = APP_NAME_MAP
            .iter()
            .find(|(exe, _)| exe.eq_ignore_ascii_case(&exe_name))
            .map(|(_, name)| name.to_string())
            .unwrap_or_else(|| {
                // Use exe filename without extension, title-cased
                exe_name
                    .strip_suffix(".exe")
                    .or_else(|| exe_name.strip_suffix(".EXE"))
                    .unwrap_or(&exe_name)
                    .to_string()
            });

        (friendly, exe_path)
    }
}

#[cfg(not(windows))]
pub fn get_foreground_app() -> (String, String) {
    ("Unknown".to_string(), String::new())
}
