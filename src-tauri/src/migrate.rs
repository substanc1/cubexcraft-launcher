// Переезд со старого лаунчера (GravitLauncher) при первом запуске: настройки из settings.json
// (userSettings.stdruntime) и сохранённые логины/пароли (remembered.dat, quick.dat - AES-GCM ключом cxc.key).
// Пароли используются один раз - для входа в новый API, сохраняются только токены сессий.
use crate::{crypto, paths};
use serde_json::{json, Value};

pub struct OldCredentials {
    pub login: String,
    pub password: String,
}

pub struct Migration {
    pub settings: Value,
    pub credentials: Vec<OldCredentials>, // первым - основной (remembered.dat), дальше быстрые аккаунты
}

pub fn read_old() -> Option<Migration> {
    let dir = paths::app_dir();
    let raw = std::fs::read_to_string(dir.join("settings.json")).ok()?;
    let old: Value = serde_json::from_str(&raw).ok()?;
    let rt = old.pointer("/userSettings/stdruntime").cloned().unwrap_or(Value::Null);
    let g = rt.get("globalSettings").cloned().unwrap_or(Value::Null);
    let mut settings = json!({
        "login": rt.get("login").cloned().unwrap_or(Value::Null),
        "autoAuth": rt.get("autoAuth").and_then(|v| v.as_bool()).unwrap_or(false),
        "locale": rt.get("locale").and_then(|v| v.as_str()).unwrap_or("RUSSIAN"),
        "lastProfile": rt.get("lastProfile").cloned().unwrap_or(Value::Null),
        "updatesDir": rt.get("updatesDirPath").cloned().unwrap_or(Value::Null),
        "debugAllClients": g.get("debugAllClients").and_then(|v| v.as_bool()).unwrap_or(true),
        "reduceEffects": g.get("reduceEffects").and_then(|v| v.as_bool()).unwrap_or(false),
        "customColors": rt.get("customColors").cloned().unwrap_or(json!({})),
        "quickAccounts": [],
        "profiles": {},
        "migratedFromGravit": true
    });
    // настройки клиентов (ОЗУ, полный экран, автовход) - backend.settings старого лаунчера
    if let Some(ps) = old.pointer("/userSettings/backend/settings").and_then(|v| v.as_object()) {
        let mut profiles = serde_json::Map::new();
        for (uuid, s) in ps {
            let flags: Vec<String> = s.get("flags").and_then(|f| f.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            profiles.insert(uuid.clone(), json!({
                "ram": s.get("ram").and_then(|v| v.as_u64()).unwrap_or(0),
                "fullScreen": flags.iter().any(|f| f == "FULLSCREEN"),
                "autoEnter": flags.iter().any(|f| f == "AUTO_ENTER"),
            }));
        }
        settings["profiles"] = Value::Object(profiles);
    }

    let mut credentials = Vec::new();
    if let Some(key) = crypto::load_key(&dir.join("cxc.key")) {
        if let Ok(rem) = std::fs::read_to_string(dir.join("remembered.dat")) {
            let lines: Vec<&str> = rem.lines().collect();
            if lines.len() >= 2 {
                if let (Some(login), Some(pass)) = (crypto::b64_decode_str(lines[0]), crypto::decrypt(&key, lines[1])) {
                    if let Ok(password) = String::from_utf8(pass) {
                        credentials.push(OldCredentials { login, password });
                    }
                }
            }
        }
        if let Ok(q) = std::fs::read_to_string(dir.join("quick.dat")) {
            for line in q.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() < 3 {
                    continue;
                }
                if let (Some(login), Some(pass)) = (crypto::b64_decode_str(parts[0]), crypto::decrypt(&key, parts[2])) {
                    if let Ok(password) = String::from_utf8(pass) {
                        if !credentials.iter().any(|c| c.login.eq_ignore_ascii_case(&login)) {
                            credentials.push(OldCredentials { login, password });
                        }
                    }
                }
            }
        }
    }
    Some(Migration { settings, credentials })
}
