// Настройки (cxl/settings.json, форму задаёт интерфейс) и сохранённые сессии (cxl/accounts.dat, зашифровано).
// Пароли не храним: «сохранить пароль» = сохранённый токен сессии (его можно отозвать на сервере).
use crate::crypto;
use crate::paths;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedAccount {
    pub login: String,
    pub username: String,
    pub uuid: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Accounts {
    pub accounts: Vec<SavedAccount>,
}

pub struct Store {
    dir: PathBuf,
    key: [u8; 32],
}

impl Store {
    pub fn open() -> Store {
        let dir = paths::cxl_dir();
        let _ = std::fs::create_dir_all(&dir);
        let key = crypto::load_or_create_key(&dir.join("key.bin"));
        Store { dir, key }
    }

    pub fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    pub fn has_settings(&self) -> bool {
        self.settings_path().exists()
    }

    pub fn load_settings(&self) -> Value {
        let v = std::fs::read_to_string(self.settings_path())
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
        with_defaults(v)
    }

    pub fn save_settings(&self, v: &Value) -> std::io::Result<()> {
        let tmp = self.dir.join("settings.json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(v).unwrap_or_default())?;
        std::fs::rename(tmp, self.settings_path())
    }

    pub fn load_accounts(&self) -> Accounts {
        std::fs::read_to_string(self.dir.join("accounts.dat"))
            .ok()
            .and_then(|s| crypto::decrypt(&self.key, &s))
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save_accounts(&self, a: &Accounts) {
        let enc = crypto::encrypt(&self.key, &serde_json::to_vec(a).unwrap_or_default());
        let tmp = self.dir.join("accounts.dat.tmp");
        if std::fs::write(&tmp, enc).is_ok() {
            let _ = std::fs::rename(tmp, self.dir.join("accounts.dat"));
        }
    }
}

/// значения по умолчанию - как RuntimeSettings.getDefault / GlobalSettings старого лаунчера
pub fn with_defaults(mut v: Value) -> Value {
    if !v.is_object() {
        v = json!({});
    }
    let o = v.as_object_mut().unwrap();
    let defaults = json!({
        "login": null,
        "autoAuth": false,
        "locale": "RUSSIAN",
        "lastProfile": null,
        "updatesDir": null,
        "debugAllClients": true,
        "reduceEffects": false,
        "customColors": {},
        "quickAccounts": [],
        "profiles": {}
    });
    for (k, d) in defaults.as_object().unwrap() {
        if !o.contains_key(k) {
            o.insert(k.clone(), d.clone());
        }
    }
    v
}

pub fn updates_dir(settings: &Value) -> PathBuf {
    settings
        .get("updatesDir")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(paths::default_updates_dir)
}
