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

/// папка клиента под эту машину: horizon.dirs[<os>-<arch>], иначе dir профиля
pub fn client_dir_name(profile: &Value) -> Result<String, String> {
    let platform = crate::java::platform();
    if let Some(dirs) = profile["horizon"]["dirs"].as_object() {
        return dirs.get(&platform).and_then(|v| v.as_str()).map(str::to_string).ok_or_else(|| {
            let title = profile["title"].as_str().unwrap_or("Эта сборка");
            format!("{title} пока не работает на этой системе ({platform})")
        });
    }
    profile["dir"].as_str().map(str::to_string).ok_or_else(|| "dir".to_string())
}

pub fn exe(client_dir: &Path) -> PathBuf {
    client_dir.join(if cfg!(windows) { "horizon-client.exe" } else { "horizon-client" })
}

/// файл сессии для --session-file: клиент принимает только свежий (до 2 минут) и, на macOS/Linux, только
/// с правами владельца. Лаунчер удаляет его вскоре после запуска - токен остаётся лишь в памяти игры.
pub fn write_session(gs: &GameSession) -> Result<PathBuf, String> {
    let dir = paths::cxl_dir().join("sessions");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
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
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path).map_err(|e| e.to_string())?;
        f.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn args(profile: &Value, resources: &Path, session: &Path, auto_enter: bool) -> Vec<String> {
    let mut args = vec![
        "--resources".to_string(), resources.to_string_lossy().to_string(),
        "--session-file".to_string(), session.to_string_lossy().to_string(),
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
        ("HORIZON_HOME".to_string(), client_dir.to_string_lossy().to_string()),
        ("HORIZON_TELEMETRY_URL".to_string(), "off".to_string()),
        ("HORIZON_NO_UPDATE_CHECK".to_string(), "1".to_string()),
    ];
    if let Some(d) = profile["horizon"]["discovery"].as_str() {
        env.push(("HORIZON_API_DISCOVERY".to_string(), d.to_string()));
    }
    env
}
