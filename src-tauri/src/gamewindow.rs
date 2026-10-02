// Окно игры: размер под экран игрока и вывод на передний план, когда оно появится.

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

#[cfg(not(windows))]
pub fn bring_to_front_when_shown(_pid: u32) {}

/// Windows не даёт процессу перехватить фокус («foreground lock»): окно игры открывалось под лаунчером и
/// браузером и только мигало в панели задач. Лаунчер ждёт окно игры (класс GLFW30) и выводит его вперёд,
/// пока оно действительно не станет активным (до 3 попыток). Поверх всех навсегда не закрепляется.
/// Что произошло - пишется в cxl/logs/game-window.log.
#[cfg(windows)]
pub fn bring_to_front_when_shown(pid: u32) {
    use std::io::Write;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let start = Instant::now();
    let path = crate::paths::cxl_dir().join("logs").join("game-window.log");
    let file = std::sync::Mutex::new(std::fs::File::create(&path).ok());
    let log = move |msg: String| {
        if let Some(f) = file.lock().unwrap().as_mut() {
            let _ = writeln!(f, "[{:6.1}s] {msg}", start.elapsed().as_secs_f32());
        }
    };
    unsafe {
        AllowSetForegroundWindow(pid);
    }
    log(format!("игра pid {pid}, ждём окно"));
    std::thread::spawn(move || {
        while start.elapsed() < Duration::from_secs(300) {
            std::thread::sleep(Duration::from_millis(200));
            let Some(h) = find_game_window(pid) else { continue };
            log(format!("окно найдено: {}", describe(h)));
            crate::timing::mark("окно игры появилось");
            std::thread::sleep(Duration::from_millis(300));
            for attempt in 1..=3 {
                unsafe {
                    if IsWindow(h) == 0 {
                        log("окно закрылось".into());
                        return;
                    }
                }
                let before = unsafe { GetForegroundWindow() };
                let how = unsafe { force_foreground(h) };
                std::thread::sleep(Duration::from_millis(700));
                let now = unsafe { GetForegroundWindow() };
                log(format!("попытка {attempt}: было активно {}, {how}, сейчас активно {}", describe(before), describe(now)));
                if now == h {
                    return;
                }
                std::thread::sleep(Duration::from_millis(1500));
            }
            return;
        }
        log("окно игры не появилось за 5 минут".into());
    });
}

/// Вперёд по очереди, пока не получится: z-порядок (TOPMOST -> NOTOPMOST), SetForegroundWindow из потока,
/// присоединённого к вводу активного окна, затем Alt (Windows снимает блокировку, когда «пользователь
/// переключает окна»), затем SwitchToThisWindow (как Alt+Tab).
#[cfg(windows)]
unsafe fn force_foreground(h: windows_sys::Win32::Foundation::HWND) -> String {
    use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_MENU};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    if IsIconic(h) != 0 {
        ShowWindow(h, SW_RESTORE);
    }
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW;
    SetWindowPos(h, HWND_TOPMOST, 0, 0, 0, 0, flags);
    SetWindowPos(h, HWND_NOTOPMOST, 0, 0, 0, 0, flags);

    let fg = GetForegroundWindow();
    let fg_thread = if fg.is_null() { 0 } else { GetWindowThreadProcessId(fg, std::ptr::null_mut()) };
    let me = GetCurrentThreadId();
    let attached = fg_thread != 0 && fg_thread != me && AttachThreadInput(me, fg_thread, 1) != 0;
    BringWindowToTop(h);
    let ok = SetForegroundWindow(h) != 0;
    if attached {
        AttachThreadInput(me, fg_thread, 0);
    }
    if ok && GetForegroundWindow() == h {
        return format!("SetForegroundWindow (attach={attached})");
    }
    keybd_event(VK_MENU as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
    keybd_event(VK_MENU as u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
    if SetForegroundWindow(h) != 0 && GetForegroundWindow() == h {
        return format!("Alt + SetForegroundWindow (attach={attached}, без Alt={ok})");
    }
    SwitchToThisWindow(h, 1);
    format!("SwitchToThisWindow (attach={attached}, без Alt={ok})")
}

/// главное окно игры: видимое окно верхнего уровня процесса, класс GLFW30 (вспомогательное «GLFW3 Helper» скрыто)
#[cfg(windows)]
fn find_game_window(pid: u32) -> Option<windows_sys::Win32::Foundation::HWND> {
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
        if owner_pid == s.pid && IsWindowVisible(h) != 0 && GetWindow(h, GW_OWNER).is_null() && class_of(h) == "GLFW30" {
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

#[cfg(windows)]
fn class_of(h: windows_sys::Win32::Foundation::HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(h, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

/// для журнала: класс, pid и заголовок окна
#[cfg(windows)]
fn describe(h: windows_sys::Win32::Foundation::HWND) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    if h.is_null() {
        return "нет окна".into();
    }
    let mut pid = 0u32;
    let mut title = [0u16; 128];
    let n = unsafe {
        GetWindowThreadProcessId(h, &mut pid);
        GetWindowTextW(h, title.as_mut_ptr(), title.len() as i32)
    };
    format!("[{} pid {pid} «{}»]", class_of(h), String::from_utf16_lossy(&title[..n.max(0) as usize]))
}
