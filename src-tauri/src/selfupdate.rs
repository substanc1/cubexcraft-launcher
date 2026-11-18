// Самообновление лаунчера (свой exe).
// Манифест: https://www.cubexcraft.site/cxl/launcher/<платформа>.json {version,url,sha256,size};
// новая версия качается рядом, текущий exe переименовывается в .old (запущенный exe на Windows переименовать
// можно, удалить - нет), новая ставится на его место и запускается; .old удаляется при следующем старте.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(windows)]
fn wait_pid(pid: u32, timeout: Duration) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
    };
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

/// Вызывается до запуска окна: ждёт выхода процесса из --wait-pid и убирает хвосты прошлого обновления.
pub fn on_startup() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--wait-pid") {
        if let Some(pid) = args.get(i + 1).and_then(|p| p.parse::<u32>().ok()) {
            wait_pid(pid, Duration::from_secs(10));
        }
    }
    let Ok(me) = std::env::current_exe() else {
        return;
    };
    let old = PathBuf::from(format!("{}.old", me.to_string_lossy()));
    if old.exists() {
        let _ = std::fs::remove_file(&old);
    }
    #[cfg(target_os = "macos")]
    if let Some(b) = bundle_of(&me) {
        let _ = std::fs::remove_dir_all(format!("{}.old", b.to_string_lossy()));
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
    } else if cfg!(target_os = "macos") {
        // архив всего CubeXCraft.app (universal): подпись бандла меняется целиком, не ломается
        "macos-app".into()
    } else {
        crate::java::platform()
    }
}

fn newer(remote: &str, local: &str) -> bool {
    let p = |s: &str| {
        s.split('.')
            .map(|x| x.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    p(remote) > p(local)
}

pub async fn check(http: &reqwest::Client) -> Option<Release> {
    let r = http
        .get(format!(
            "{}/cxl/launcher/{}.json",
            crate::api::SITE,
            platform_key()
        ))
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
pub async fn apply(
    http: &reqwest::Client,
    rel: &Release,
    emit: &(dyn Fn(u64, u64) + Send + Sync),
) -> Result<(), String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = if cfg!(target_os = "macos") {
        std::env::temp_dir().join(format!("cubexcraft-update-{}.tar.gz", std::process::id()))
    } else {
        PathBuf::from(format!("{}.download", me.to_string_lossy()))
    };
    let r = http
        .get(&rel.url)
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    if !r.status().is_success() {
        return Err(format!("HTTP {}", r.status().as_u16()));
    }
    let mut f = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| e.to_string())?;
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
    #[cfg(target_os = "macos")]
    {
        let r = apply_mac_bundle(&me, &tmp);
        let _ = std::fs::remove_file(&tmp);
        return r;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755));
    }
    #[allow(unreachable_code)]
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

/// .app, внутри которого мы запущены (…/CubeXCraft.app/Contents/MacOS/CubeX)
#[cfg(target_os = "macos")]
fn bundle_of(exe: &Path) -> Option<PathBuf> {
    exe.ancestors()
        .find(|p| p.extension().map(|e| e == "app").unwrap_or(false))
        .map(Path::to_path_buf)
}

/// macOS: распаковать архив .app рядом с текущим (тот же том), старый -> .app.old, новый на его место,
/// запуск нового через LaunchServices. .app.old удаляется при следующем старте (on_startup).
#[cfg(target_os = "macos")]
fn apply_mac_bundle(me: &Path, archive: &Path) -> Result<(), String> {
    let bundle = bundle_of(me).ok_or("лаунчер запущен не из .app")?;
    let parent = bundle.parent().ok_or("нет папки приложения")?.to_path_buf();
    let staging = parent.join(format!(".cubexcraft-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("{}: {e}", parent.display()))?;
    let f = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    tar::Archive::new(flate2::read::GzDecoder::new(f))
        .unpack(&staging)
        .map_err(|e| e.to_string())?;
    let new_app = std::fs::read_dir(&staging)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().map(|e| e == "app").unwrap_or(false))
        .ok_or("в архиве обновления нет .app")?;
    let old = PathBuf::from(format!("{}.old", bundle.to_string_lossy()));
    let _ = std::fs::remove_dir_all(&old);
    std::fs::rename(&bundle, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&new_app, &bundle) {
        let _ = std::fs::rename(&old, &bundle);
        let _ = std::fs::remove_dir_all(&staging);
        return Err(e.to_string());
    }
    let _ = std::fs::remove_dir_all(&staging);
    let _ = std::process::Command::new("open")
        .arg("-n")
        .arg(&bundle)
        .arg("--args")
        .arg("--wait-pid")
        .arg(std::process::id().to_string())
        .spawn();
    Ok(())
}
