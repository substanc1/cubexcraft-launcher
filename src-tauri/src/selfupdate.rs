// Самообновление.
// 1) Переезд со старого лаунчера: его LauncherSelfUpdate качает CubeX.exe (это уже мы) в
//    %APPDATA%\CubeXCraft\CubeX.exe.<pid>.new, пишет pending-launcher-update.json {target,newFile,oldPid} и запускает
//    скачанный файл. Мы при старте видим метку, ждём выхода старого процесса, копируем себя поверх target (ярлык
//    игрока), удаляем метку и перезапускаемся уже оттуда.
// 2) Свои обновления: https://www.cubexcraft.site/cxl/launcher/<платформа>.json {version,url,sha256,size};
//    новая версия качается рядом, текущий exe переименовывается в .old (запущенный exe на Windows переименовать
//    можно, удалить - нет), новая ставится на его место и запускается; .old удаляется при следующем старте.
use crate::paths;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MARKER: &str = "pending-launcher-update.json";

#[derive(Deserialize)]
struct GravitMarker {
    target: String,
    #[serde(rename = "newFile")]
    new_file: String,
    #[serde(rename = "oldPid")]
    old_pid: Option<String>,
}

fn same_path(a: &Path, b: &Path) -> bool {
    let ca = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    if cfg!(windows) {
        ca.to_string_lossy().to_lowercase() == cb.to_string_lossy().to_lowercase()
    } else {
        ca == cb
    }
}

#[cfg(windows)]
fn wait_pid(pid: u32, timeout: Duration) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE};
    unsafe {
        let h = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !h.is_null() {
            WaitForSingleObject(h, timeout.as_millis() as u32);
            CloseHandle(h);
        }
    }
}
#[cfg(not(windows))]
fn wait_pid(_pid: u32, timeout: Duration) {
    std::thread::sleep(timeout.min(Duration::from_secs(2)));
}

/// новый процесс сначала ждёт выхода текущего (иначе защита от второго экземпляра закроет его)
fn spawn_detached(exe: &Path) {
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--wait-pid").arg(std::process::id().to_string());
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    let _ = cmd.spawn();
}

/// Вызывается до запуска окна. true = процесс должен завершиться (перезапуск уже сделан).
pub fn on_startup() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--wait-pid") {
        if let Some(pid) = args.get(i + 1).and_then(|p| p.parse::<u32>().ok()) {
            wait_pid(pid, Duration::from_secs(10));
        }
    }
    let Ok(me) = std::env::current_exe() else { return false };
    // хвосты прошлого обновления
    let old = PathBuf::from(format!("{}.old", me.to_string_lossy()));
    if old.exists() {
        let _ = std::fs::remove_file(&old);
    }
    let marker = paths::app_dir().join(MARKER);
    let Ok(raw) = std::fs::read_to_string(&marker) else { return false };
    let Ok(m) = serde_json::from_str::<GravitMarker>(&raw) else {
        let _ = std::fs::remove_file(&marker);
        return false;
    };
    let target = PathBuf::from(&m.target);
    let new_file = PathBuf::from(&m.new_file);
    if !same_path(&me, &new_file) {
        // мы - уже установленный лаунчер: метка устарела, подчищаем скачанные .new
        if same_path(&me, &target) {
            let _ = std::fs::remove_file(&marker);
            cleanup_new_files(&target);
        }
        return false;
    }
    if let Some(pid) = m.old_pid.as_deref().and_then(|p| p.parse::<u32>().ok()) {
        wait_pid(pid, Duration::from_secs(10));
    }
    let mut ok = false;
    for _ in 0..30 {
        if std::fs::copy(&me, &target).is_ok() {
            ok = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    if ok {
        let _ = std::fs::remove_file(&marker);
        spawn_detached(&target);
        return true;
    }
    false // не вышло - работаем из .new, попробуем при следующем запуске
}

fn cleanup_new_files(target: &Path) {
    let Some(name) = target.file_name().map(|n| n.to_string_lossy().to_string()) else { return };
    for dir in [paths::app_dir(), target.parent().map(Path::to_path_buf).unwrap_or_default()] {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let f = e.file_name().to_string_lossy().to_string();
                if f.starts_with(&format!("{name}.")) && f.ends_with(".new") {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    }
}

#[derive(Deserialize, Clone, serde::Serialize)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

pub fn platform_key() -> String {
    // какой exe ставить на этой машине: Windows - всегда x64 (на ARM работает в эмуляции)
    if cfg!(windows) {
        "windows-x64".into()
    } else {
        crate::java::platform()
    }
}

fn newer(remote: &str, local: &str) -> bool {
    let p = |s: &str| s.split('.').map(|x| x.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    p(remote) > p(local)
}

pub async fn check(http: &reqwest::Client) -> Option<Release> {
    let r = http
        .get(format!("{}/cxl/launcher/{}.json", crate::api::SITE, platform_key()))
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .ok()?;
    if !r.status().is_success() {
        return None;
    }
    let rel: Release = r.json().await.ok()?;
    newer(&rel.version, env!("CARGO_PKG_VERSION")).then_some(rel)
}

/// скачать, проверить, поставить на место текущего exe и запустить; emit(done,total)
pub async fn apply(http: &reqwest::Client, rel: &Release, emit: &(dyn Fn(u64, u64) + Send + Sync)) -> Result<(), String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = PathBuf::from(format!("{}.download", me.to_string_lossy()));
    let r = http.get(&rel.url).send().await.map_err(|e| e.without_url().to_string())?;
    if !r.status().is_success() {
        return Err(format!("HTTP {}", r.status().as_u16()));
    }
    let mut f = tokio::fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut got = 0u64;
    let mut s = r.bytes_stream();
    while let Some(c) = s.next().await {
        let c = c.map_err(|e| e.without_url().to_string())?;
        h.update(&c);
        f.write_all(&c).await.map_err(|e| e.to_string())?;
        got += c.len() as u64;
        emit(got, rel.size);
    }
    f.flush().await.map_err(|e| e.to_string())?;
    drop(f);
    if hex::encode(h.finalize()) != rel.sha256.to_lowercase() {
        let _ = std::fs::remove_file(&tmp);
        return Err("контрольная сумма обновления не совпала".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755));
    }
    let old = PathBuf::from(format!("{}.old", me.to_string_lossy()));
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&me, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&tmp, &me) {
        let _ = std::fs::rename(&old, &me);
        return Err(e.to_string());
    }
    spawn_detached(&me);
    Ok(())
}
