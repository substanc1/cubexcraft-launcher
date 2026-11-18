// Профили на движке Horizon (Minecraft 26.3 на Rust, проект CubeXRust): вместо Java - свой клиент
// horizon-client из папки профиля, данные игры (подготовленные из 26.3) - из assetDir профиля. Вход - как у
// Java-профилей: игровой токен сайта, только передаётся не аргументами, а файлом сессии (--session-file),
// а адреса join/hasJoined клиент берёт из discovery-документа сайта (HORIZON_API_DISCOVERY).
use crate::{api::GameSession, paths};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn is_horizon(profile: &Value) -> bool {
    profile["engine"].as_str() == Some("horizon")
}

/// ключ платформы для horizon.dirs: os ("windows"|"macos"|"linux") + "-" + ("arm64"|"x64") по машине
pub fn platform_key() -> String {
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if paths::machine_arch() == "arm64" {
        "arm64"
    } else {
        "x64"
    };
    format!("{os}-{arch}")
}

/// текст ошибки, когда для платформы нет пакета: «<title> пока не поддерживается на <ОС> <арх>»
pub fn unsupported_msg(title: &str, platform: &str) -> String {
    let title = if title.trim().is_empty() {
        "CubeXRust"
    } else {
        title.trim()
    };
    let (os, arch) = platform.split_once('-').unwrap_or((platform, ""));
    let os = match os {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    };
    let arch = match arch {
        "x64" => "x64",
        "arm64" => "ARM64",
        other => other,
    };
    let system = if arch.is_empty() {
        os.to_string()
    } else {
        format!("{os} {arch}")
    };
    format!("{title} пока не поддерживается на {system}")
}

/// папка клиента для платформы: horizon.dirs[platform]; если horizon.dirs нет - dir профиля
pub fn pick_dir(profile: &Value, platform: &str) -> Result<String, String> {
    if let Some(dirs) = profile["horizon"]["dirs"].as_object() {
        return dirs
            .get(platform)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| unsupported_msg(profile["title"].as_str().unwrap_or(""), platform));
    }
    profile["dir"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "dir".to_string())
}

/// папка клиента под эту машину (см. pick_dir)
pub fn client_dir_name(profile: &Value) -> Result<String, String> {
    pick_dir(profile, &platform_key())
}

pub fn exe_name(windows: bool) -> &'static str {
    if windows {
        "horizon-client.exe"
    } else {
        "horizon-client"
    }
}

pub fn exe(client_dir: &Path) -> PathBuf {
    client_dir.join(exe_name(cfg!(windows)))
}

/// подготовка exe к запуску: права 0o755 (unix); на macOS снимаем quarantine только с exe. Ошибки игнорируем.
pub fn prepare_exe(exe: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(exe, std::fs::Permissions::from_mode(0o755));
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("xattr")
            .args(["-dr", "com.apple.quarantine"])
            .arg(exe)
            .output();
    }
    #[cfg(not(unix))]
    let _ = exe;
}

/// файл сессии для --session-file: клиент принимает только свежий (до 2 минут) и, на macOS/Linux, только
/// с правами владельца. Лаунчер удаляет его вскоре после запуска - токен остаётся лишь в памяти игры.
pub fn write_session(gs: &GameSession) -> Result<PathBuf, String> {
    let dir = paths::cxl_dir().join("sessions");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("horizon-{stamp}.json"));
    let body = serde_json::json!({
        "name": gs.username,
        "uuid": gs.uuid,
        "access_token": gs.access_token,
        "user_type": "msa",
    })
    .to_string();
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .map_err(|e| e.to_string())?;
        f.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn args(profile: &Value, resources: &Path, session: &Path, auto_enter: bool) -> Vec<String> {
    let mut args = vec![
        "--resources".to_string(),
        resources.to_string_lossy().to_string(),
        "--session-file".to_string(),
        session.to_string_lossy().to_string(),
    ];
    if auto_enter {
        if let Some(s) = profile["servers"].as_array().and_then(|a| a.first()) {
            let host = s["serverAddress"].as_str().unwrap_or("");
            let port = s["serverPort"].as_u64().unwrap_or(25565);
            args.push("--server".into());
            args.push(format!("{host}:{port}"));
        }
    }
    args
}

/// окружение игры: её папка, адреса входа сайта, без телеметрии и проверки обновлений автора Horizon
pub fn env(profile: &Value, client_dir: &Path) -> Vec<(String, String)> {
    let mut env = vec![
        (
            "HORIZON_HOME".to_string(),
            client_dir.to_string_lossy().to_string(),
        ),
        ("HORIZON_TELEMETRY_URL".to_string(), "off".to_string()),
        ("HORIZON_NO_UPDATE_CHECK".to_string(), "1".to_string()),
    ];
    if let Some(d) = profile["horizon"]["discovery"].as_str() {
        env.push(("HORIZON_API_DISCOVERY".to_string(), d.to_string()));
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn profile_with_dirs() -> Value {
        json!({"title": "CubeXRust", "dir": "cubexrust", "horizon": {"dirs": {
            "windows-x64": "cubexrust-win-x64",
            "macos-arm64": "cubexrust-mac-arm64",
        }}})
    }

    #[test]
    fn pick_dir_windows_x64() {
        assert_eq!(
            pick_dir(&profile_with_dirs(), "windows-x64").unwrap(),
            "cubexrust-win-x64"
        );
    }

    #[test]
    fn pick_dir_macos_arm64() {
        assert_eq!(
            pick_dir(&profile_with_dirs(), "macos-arm64").unwrap(),
            "cubexrust-mac-arm64"
        );
    }

    #[test]
    fn pick_dir_missing_key_is_error() {
        let p = profile_with_dirs();
        let e = pick_dir(&p, "linux-x64").unwrap_err();
        assert!(e.contains("пока не поддерживается на"), "{e}");
        assert_eq!(e, "CubeXRust пока не поддерживается на Linux x64");
    }

    #[test]
    fn pick_dir_macos_arm64_missing_mentions_macos_arm64() {
        let p = json!({"title": "CubeXRust", "dir": "cubexrust", "horizon": {"dirs": {"windows-x64": "A"}}});
        let e = pick_dir(&p, "macos-arm64").unwrap_err();
        assert!(e.contains("пока не поддерживается на"), "{e}");
        assert!(e.contains("macOS ARM64"), "{e}");
        assert_eq!(e, "CubeXRust пока не поддерживается на macOS ARM64");
    }

    #[test]
    fn unsupported_msg_default_title() {
        assert_eq!(
            unsupported_msg("", "windows-x64"),
            "CubeXRust пока не поддерживается на Windows x64"
        );
        assert_eq!(
            unsupported_msg("  ", "linux-arm64"),
            "CubeXRust пока не поддерживается на Linux ARM64"
        );
    }

    #[test]
    fn pick_dir_without_horizon_dirs_uses_dir() {
        let p = json!({"title": "X", "dir": "cubexrust", "engine": "horizon"});
        assert_eq!(pick_dir(&p, "macos-arm64").unwrap(), "cubexrust");
    }

    #[test]
    fn exe_name_by_os() {
        assert_eq!(exe_name(true), "horizon-client.exe");
        assert_eq!(exe_name(false), "horizon-client");
    }

    #[test]
    fn platform_key_shape() {
        let k = platform_key();
        assert!(
            [
                "windows-x64",
                "windows-arm64",
                "macos-x64",
                "macos-arm64",
                "linux-x64",
                "linux-arm64"
            ]
            .contains(&k.as_str()),
            "{k}"
        );
    }

    #[test]
    fn is_horizon_only_for_engine_horizon() {
        assert!(is_horizon(&json!({"engine": "horizon"})));
        assert!(!is_horizon(&json!({"engine": "java"})));
        assert!(!is_horizon(&json!({"dir": "x"})));
    }
}
