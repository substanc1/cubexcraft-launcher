// Java для игры: одна загрузка с нашего сервера (а не 117 МБ Java+JavaFX с BellSoft раз в 30 дней, как у
// старого престартера). Описание сборки: https://www.cubexcraft.site/cxl/java/<платформа>.json
//   { "version": "...", "url": "...", "sha1": "...", "size": N, "format": "zip"|"tar.gz" }
// Распакованная Java лежит в cxl/java/<платформа>/, рядом метка .cxl-java.json с версией.
use crate::{api, paths};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct JavaBuild {
    pub version: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub format: String,
}

pub fn platform() -> String {
    let os = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" };
    let arch = if paths::game_arch() == "arm64" { "arm64" } else { "x64" };
    format!("{os}-{arch}")
}

fn java_exe(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    }
}

/// корень JAVA_HOME внутри распакованного архива (на macOS - Contents/Home)
fn find_home(dir: &Path) -> Option<PathBuf> {
    if java_exe(dir).exists() {
        return Some(dir.to_path_buf());
    }
    let mac = dir.join("Contents").join("Home");
    if java_exe(&mac).exists() {
        return Some(mac);
    }
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        if e.path().is_dir() {
            if let Some(h) = find_home(&e.path()) {
                return Some(h);
            }
        }
    }
    None
}

pub fn installed() -> Option<(String, PathBuf)> {
    let dir = paths::cxl_dir().join("java").join(platform());
    let marker: JavaBuild = serde_json::from_slice(&std::fs::read(dir.join(".cxl-java.json")).ok()?).ok()?;
    let home = find_home(&dir.join("jre"))?;
    Some((marker.version, java_exe(&home)))
}

/// Вернуть путь к java. emit(done,total) - прогресс загрузки.
pub async fn ensure(http: &reqwest::Client, emit: &(dyn Fn(u64, u64) + Send + Sync)) -> Result<PathBuf, String> {
    let current = installed();
    let remote: Result<JavaBuild, String> = async {
        let r = http
            .get(format!("{}/cxl/java/{}.json", api::SITE, platform()))
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?;
        if !r.status().is_success() {
            return Err(format!("HTTP {}", r.status().as_u16()));
        }
        r.json::<JavaBuild>().await.map_err(|e| e.to_string())
    }
    .await;
    let remote = match (remote, &current) {
        (Ok(r), _) => r,
        (Err(_), Some((_, exe))) => return Ok(exe.clone()), // нет связи - работаем с той, что есть
        (Err(e), None) => return Err(format!("Не удалось получить Java для {}: {e}", platform())),
    };
    if let Some((ver, exe)) = &current {
        if *ver == remote.version && exe.exists() {
            return Ok(exe.clone());
        }
    }
    let dir = paths::cxl_dir().join("java").join(platform());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let archive = dir.join(format!("download.{}", remote.format));
    // загрузка
    {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;
        let r = http.get(&remote.url).send().await.map_err(|e| e.without_url().to_string())?;
        if !r.status().is_success() {
            return Err(format!("Java: HTTP {}", r.status().as_u16()));
        }
        let mut f = tokio::fs::File::create(&archive).await.map_err(|e| e.to_string())?;
        let mut h = Sha1::new();
        let mut got = 0u64;
        let mut s = r.bytes_stream();
        while let Some(c) = s.next().await {
            let c = c.map_err(|e| e.without_url().to_string())?;
            h.update(&c);
            f.write_all(&c).await.map_err(|e| e.to_string())?;
            got += c.len() as u64;
            emit(got, remote.size);
        }
        f.flush().await.map_err(|e| e.to_string())?;
        if hex::encode(h.finalize()) != remote.sha1 {
            let _ = std::fs::remove_file(&archive);
            return Err("Java: контрольная сумма архива не совпала".into());
        }
    }
    // распаковка в jre.tmp -> jre
    let tmp = dir.join("jre.tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    let fmt = remote.format.clone();
    let a2 = archive.clone();
    let t2 = tmp.clone();
    tokio::task::spawn_blocking(move || extract(&a2, &t2, &fmt)).await.map_err(|e| e.to_string())??;
    let final_dir = dir.join("jre");
    let _ = std::fs::remove_dir_all(&final_dir);
    std::fs::rename(&tmp, &final_dir).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&archive);
    std::fs::write(dir.join(".cxl-java.json"), serde_json::to_vec(&remote).unwrap_or_default()).map_err(|e| e.to_string())?;
    let home = find_home(&final_dir).ok_or("Java: в архиве нет bin/java")?;
    Ok(java_exe(&home))
}

fn extract(archive: &Path, to: &Path, format: &str) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    let f = std::fs::File::open(archive).map_err(|e| e.to_string())?;
    if format == "zip" {
        let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
        z.extract(to).map_err(|e| e.to_string())?;
    } else {
        let gz = flate2::read::GzDecoder::new(f);
        tar::Archive::new(gz).unpack(to).map_err(|e| e.to_string())?;
    }
    Ok(())
}
