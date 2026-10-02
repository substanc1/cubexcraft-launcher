// Уборка за старым лаунчером (GravitLauncher) - один раз, после первого успешного входа в новом лаунчере
// (к этому моменту настройки и аккаунты уже перенесены, см. migrate.rs):
//  * Java престартера: <data_dir>/GravitLauncherStore (JRE ~300 МБ, перекачивалась раз в 30 дней);
//  * кэш WebView престартера: com.gravitlauncher.prestarter;
//  * файлы старого лаунчера в папке CubeXCraft: settings.json, сохранённые пароли (remembered.dat, quick.dat,
//    ключ cxc.key), ключи/замки/отладочные логи, недокачанные .new самообновления.
// Папка updates (скачанные клиенты, миры, скриншоты) не трогается.
use crate::paths;
use std::path::PathBuf;

fn data_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    #[cfg(windows)]
    {
        if let Some(a) = std::env::var_os("APPDATA") {
            v.push(PathBuf::from(a));
        }
        if let Some(a) = std::env::var_os("LOCALAPPDATA") {
            v.push(PathBuf::from(a));
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(h) = std::env::var_os("HOME").map(PathBuf::from) {
            v.push(std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| h.join(".local/share")));
            v.push(h.join("Library/Application Support"));
            v.push(h.join("Library/Caches"));
            v.push(h.join("Library/WebKit"));
        }
    }
    v
}

pub fn run() {
    for base in data_dirs() {
        for name in ["GravitLauncherStore", "com.gravitlauncher.prestarter"] {
            let p = base.join(name);
            if p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            }
        }
    }
    let app = paths::app_dir();
    for f in [
        "settings.json", "remembered.dat", "quick.dat", "cxc.key", "private.key", "public.key", "launcher.lock",
        "self-update.lock", "pending-launcher-update.json", "close-debug.txt", "dock-icon-debug.txt", "quick-debug.txt",
    ] {
        let _ = std::fs::remove_file(app.join(f));
    }
    if let Ok(rd) = std::fs::read_dir(&app) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            // недокачанные/старые файлы самообновления Gravit и его темы-CSS
            if (n.ends_with(".new") && n.contains(".exe.")) || n.starts_with("cubexcraft-custom-theme-") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}
