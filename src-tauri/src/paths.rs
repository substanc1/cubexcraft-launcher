// Каталоги лаунчера - те же, что у старого (GravitLauncher DirBridge.getAppDataDir + projectName),
// чтобы подхватить уже скачанные клиенты (updates/) и данные без повторной загрузки:
//   Windows: %APPDATA%\CubeXCraft,  Linux: ~/.minecraftlauncher/CubeXCraft,  macOS: ~/minecraft/CubeXCraft
use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn app_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(a) = std::env::var_os("APPDATA") {
            return PathBuf::from(a).join("CubeXCraft");
        }
        return home().join("AppData").join("Roaming").join("CubeXCraft");
    }
    #[cfg(target_os = "macos")]
    {
        return home().join("minecraft").join("CubeXCraft");
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return home().join(".minecraftlauncher").join("CubeXCraft");
    }
}

/// свои файлы нового лаунчера (настройки, сессии, кэш хэшей, java, библиотеки) - отдельно от файлов старого
pub fn cxl_dir() -> PathBuf {
    app_dir().join("cxl")
}

pub fn default_updates_dir() -> PathBuf {
    app_dir().join("updates")
}

/// архитектура игры (Java + natives). Windows - всегда x64, как у старого лаунчера (на ARM - эмуляция x64;
/// нативную ARM64-Java под Parallels/OpenGL ещё не проверяли). Остальные ОС - архитектура машины.
pub fn game_arch() -> &'static str {
    if cfg!(windows) {
        "x86-64"
    } else {
        machine_arch()
    }
}

/// os/arch в терминах папки natives клиента (Gravit JVMHelper: mustdie|linux|macosx / x86-64|arm64)
pub fn natives_os() -> &'static str {
    if cfg!(windows) {
        "mustdie"
    } else if cfg!(target_os = "macos") {
        "macosx"
    } else {
        "linux"
    }
}

/// архитектура МАШИНЫ (не процесса лаунчера): x64-лаунчер на Windows ARM работает в эмуляции,
/// а игру выгоднее запускать нативной ARM64-Java
pub fn machine_arch() -> &'static str {
    #[cfg(windows)]
    {
        // у эмулируемого x64-процесса PROCESSOR_ARCHITECTURE=AMD64; настоящую архитектуру машины
        // честно отдаёт IsWow64Process2 (native machine = IMAGE_FILE_MACHINE_ARM64 0xAA64)
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, IsWow64Process2};
        let mut process: u16 = 0;
        let mut native: u16 = 0;
        let ok = unsafe { IsWow64Process2(GetCurrentProcess(), &mut process, &mut native) };
        if ok != 0 && native == 0xAA64 {
            return "arm64";
        }
        return "x86-64";
    }
    #[cfg(not(windows))]
    {
        if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x86-64"
        }
    }
}
