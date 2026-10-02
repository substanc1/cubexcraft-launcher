// Окно игры: размер под экран игрока и вывод на передний план, когда оно появится.
#[cfg(windows)]
use std::time::Duration;

/// 16:9, ~72% рабочей области основного монитора (Minecraft центрирует окно на основном), не меньше 854x480.
/// Windows/Linux - физические пиксели (GLFW в Minecraft DPI-aware), macOS - точки.
pub fn size(app: &tauri::AppHandle) -> (u32, u32) {
    use tauri::Manager;
    let mon = app.get_webview_window("main").and_then(|w| w.primary_monitor().ok().flatten());
    let Some(mon) = mon else { return (1280, 720) };
    let area = mon.work_area();
    let (mut aw, mut ah) = (area.size.width as f64, area.size.height as f64);
    if cfg!(target_os = "macos") {
        let s = mon.scale_factor().max(1.0);
        aw /= s;
        ah /= s;
    }
    let w = (aw * 0.72).min(ah * 0.72 * 16.0 / 9.0).max(854f64.min(aw * 0.95));
    let w = (w as u32) & !1;
    (w, ((w as f64) * 9.0 / 16.0).round() as u32)
}

/// Windows не даёт новому процессу перехватить фокус, если игрок успел кликнуть куда-то ещё - окно игры
/// открывалось под лаунчером или браузером и только мигало в панели задач. Лаунчер разрешает игре фокус и сам
/// выводит её окно (класс GLFW*) вперёд, как только оно появится. Поверх всех навсегда не закрепляется.
#[cfg(windows)]
pub fn bring_to_front_when_shown(pid: u32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        AllowSetForegroundWindow(pid);
    }
    std::thread::spawn(move || {
        for _ in 0..900 {
            std::thread::sleep(Duration::from_millis(200));
            let Some(h) = find_glfw_window(pid) else { continue };
            std::thread::sleep(Duration::from_millis(300));
            unsafe {
                if IsIconic(h) != 0 {
                    ShowWindow(h, SW_RESTORE);
                }
                let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW;
                SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, flags);
                SetWindowPos(h, HWND_NOTOPMOST, 0, 0, 0, 0, flags);
                BringWindowToTop(h);
                SetForegroundWindow(h);
            }
            return;
        }
    });
}

#[cfg(not(windows))]
pub fn bring_to_front_when_shown(_pid: u32) {}

#[cfg(windows)]
fn find_glfw_window(pid: u32) -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    struct Search {
        pid: u32,
        found: HWND,
    }
    unsafe extern "system" fn visit(h: HWND, l: LPARAM) -> BOOL {
        let s = &mut *(l as *mut Search);
        let mut owner_pid = 0u32;
        GetWindowThreadProcessId(h, &mut owner_pid);
        if owner_pid != s.pid || IsWindowVisible(h) == 0 || !GetWindow(h, GW_OWNER).is_null() {
            return 1;
        }
        let mut buf = [0u16; 64];
        let n = GetClassNameW(h, buf.as_mut_ptr(), buf.len() as i32);
        if n > 0 && String::from_utf16_lossy(&buf[..n as usize]).starts_with("GLFW") {
            s.found = h;
            return 0;
        }
        1
    }
    let mut s = Search { pid, found: std::ptr::null_mut() };
    unsafe {
        EnumWindows(Some(visit), &mut s as *mut Search as LPARAM);
    }
    (!s.found.is_null()).then_some(s.found)
}
