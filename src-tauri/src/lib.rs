// CubeXCraft Launcher - ядро (Rust/Tauri). Интерфейс - app/src (Svelte), общение через invoke-команды и события.
mod api;
mod cleanup;
mod crypto;
mod java;
mod launch;
mod migrate;
mod paths;
mod ping;
mod selfupdate;
mod store;
mod sync;

use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use store::{SavedAccount, Store};
use tauri::{Emitter, Manager, State};

const MAX_QUICK_ACCOUNTS: usize = 3;

struct Session {
    login: String,
    token: String,
    user: api::User,
}

struct AppState {
    http: reqwest::Client,
    store: Store,
    settings: Mutex<Value>,
    session: Mutex<Option<Session>>,
    profiles: Mutex<Vec<Value>>,
    cancel: Arc<AtomicBool>,
    index: Arc<Mutex<sync::Index>>,
    game_kill: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

type R<T> = Result<T, String>;

fn user_json(u: &api::User) -> Value {
    json!({ "username": u.username, "uuid": u.uuid, "skin": u.skin, "cloak": u.cloak })
}

fn device() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

impl AppState {
    /// разовая уборка за старым лаунчером после первого успешного входа (cleanup.rs)
    fn cleanup_gravit_once(&self) {
        let mut s = self.settings.lock().unwrap();
        if s.get("gravitCleaned").and_then(|v| v.as_bool()).unwrap_or(false) {
            return;
        }
        s["gravitCleaned"] = json!(true);
        drop(s);
        self.save_settings();
        std::thread::spawn(cleanup::run);
    }

    fn save_settings(&self) {
        let s = self.settings.lock().unwrap().clone();
        let _ = self.store.save_settings(&s);
    }

    fn quick_list(&self) -> Value {
        let accounts = self.store.load_accounts();
        let s = self.settings.lock().unwrap();
        let list: Vec<Value> = s
            .get("quickAccounts")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|l| l.as_str())
            .filter_map(|l| accounts.accounts.iter().find(|a| a.login.eq_ignore_ascii_case(l)))
            .map(|a| json!({ "login": a.login, "username": a.username }))
            .collect();
        Value::Array(list)
    }

    fn remember(&self, login: &str, token: &str, user: &api::User) {
        let mut acc = self.store.load_accounts();
        acc.accounts.retain(|a| !a.login.eq_ignore_ascii_case(login));
        acc.accounts.insert(0, SavedAccount { login: login.into(), username: user.username.clone(), uuid: user.uuid.clone(), token: token.into() });
        self.store.save_accounts(&acc);
        let mut s = self.settings.lock().unwrap();
        let mut q: Vec<String> = s.get("quickAccounts").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect();
        q.retain(|l| !l.eq_ignore_ascii_case(login));
        q.insert(0, login.to_string());
        q.truncate(MAX_QUICK_ACCOUNTS);
        s["quickAccounts"] = json!(q);
        drop(s);
        // в accounts.dat - только аккаунты из быстрого списка
        let keep: Vec<String> = q;
        let mut acc = self.store.load_accounts();
        acc.accounts.retain(|a| keep.iter().any(|l| l.eq_ignore_ascii_case(&a.login)));
        self.store.save_accounts(&acc);
    }

    fn forget(&self, login: &str) {
        let mut acc = self.store.load_accounts();
        acc.accounts.retain(|a| !a.login.eq_ignore_ascii_case(login));
        self.store.save_accounts(&acc);
        let mut s = self.settings.lock().unwrap();
        let q: Vec<String> = s.get("quickAccounts").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).filter(|l| !l.eq_ignore_ascii_case(login)).collect();
        s["quickAccounts"] = json!(q);
    }

    fn token(&self) -> R<String> {
        self.session.lock().unwrap().as_ref().map(|s| s.token.clone()).ok_or_else(|| "auth.invalidtoken".to_string())
    }
}

// ---------------------------------------------------------------- старт

#[tauri::command]
async fn app_init(state: State<'_, AppState>) -> R<Value> {
    // первый запуск: перенос настроек и аккаунтов со старого лаунчера
    if !state.store.has_settings() {
        if let Some(m) = migrate::read_old() {
            *state.settings.lock().unwrap() = store::with_defaults(m.settings);
            for c in m.credentials {
                if let Ok((token, user)) = api::login(&state.http, &c.login, &c.password, &device()).await {
                    state.remember(&c.login, &token, &user);
                }
            }
            // основной аккаунт - первым в быстром списке
            let main = state.settings.lock().unwrap().get("login").and_then(|v| v.as_str()).map(String::from);
            if let Some(login) = main {
                let acc = state.store.load_accounts();
                if let Some(a) = acc.accounts.iter().find(|a| a.login.eq_ignore_ascii_case(&login)).cloned() {
                    state.remember(&a.login, &a.token, &api::User { uuid: a.uuid, username: a.username, skin: None, cloak: None });
                }
            }
        }
        state.save_settings();
    }
    let accounts = state.store.load_accounts();
    let mut s = state.settings.lock().unwrap().clone();
    let has_saved = s.get("login").and_then(|v| v.as_str()).map(|l| accounts.accounts.iter().any(|a| a.login.eq_ignore_ascii_case(l))).unwrap_or(false);
    s["hasSavedSession"] = json!(has_saved);
    Ok(json!({
        "settings": s,
        "quickAccounts": state.quick_list(),
        "version": env!("CARGO_PKG_VERSION"),
        "javaVersion": "21",
        "platform": java::platform(),
        "updatesDir": store::updates_dir(&state.settings.lock().unwrap()).to_string_lossy(),
    }))
}

#[tauri::command]
fn settings_save(state: State<'_, AppState>, settings: Value) -> R<()> {
    let mut s = store::with_defaults(settings);
    if let Some(o) = s.as_object_mut() {
        o.remove("hasSavedSession");
    }
    *state.settings.lock().unwrap() = s;
    state.save_settings();
    Ok(())
}

// ---------------------------------------------------------------- вход

#[tauri::command]
async fn auth_login(state: State<'_, AppState>, login: String, password: String) -> R<Value> {
    let (token, user) = api::login(&state.http, &login, &password, &device()).await.map_err(String::from)?;
    let out = user_json(&user);
    *state.session.lock().unwrap() = Some(Session { login, token, user });
    state.cleanup_gravit_once();
    Ok(out)
}

#[tauri::command]
async fn auth_restore(state: State<'_, AppState>, login: String) -> R<Value> {
    let acc = state.store.load_accounts();
    let a = acc.accounts.iter().find(|a| a.login.eq_ignore_ascii_case(&login)).cloned().ok_or("auth.invalidtoken")?;
    match api::me(&state.http, &a.token).await {
        Ok(user) => {
            let out = user_json(&user);
            *state.session.lock().unwrap() = Some(Session { login: a.login, token: a.token, user });
            state.cleanup_gravit_once();
            Ok(out)
        }
        Err(e) => {
            if e.0 == "auth.invalidtoken" {
                state.forget(&a.login);
                state.save_settings();
            }
            Err(e.0)
        }
    }
}

/// после входа: «сохранить пароль» = сохранить токен этой сессии (и в быстрый список аккаунтов)
#[tauri::command]
fn auth_remember(state: State<'_, AppState>, remember: bool) -> R<Value> {
    let sess = state.session.lock().unwrap().as_ref().map(|s| (s.login.clone(), s.token.clone(), s.user.clone()));
    if let Some((login, token, user)) = sess {
        if remember {
            state.remember(&login, &token, &user);
        } else {
            state.forget(&login);
        }
        state.save_settings();
    }
    Ok(state.quick_list())
}

#[tauri::command]
async fn auth_logout(state: State<'_, AppState>) -> R<Value> {
    let sess = state.session.lock().unwrap().take();
    if let Some(s) = sess {
        api::logout(&state.http, &s.token).await;
        state.forget(&s.login);
        state.save_settings();
    }
    Ok(state.quick_list())
}

#[tauri::command]
fn accounts_list(state: State<'_, AppState>) -> Value {
    state.quick_list()
}

// ---------------------------------------------------------------- профили и серверы

#[tauri::command]
async fn profiles_list(state: State<'_, AppState>) -> R<Vec<Value>> {
    let token = state.token()?;
    let mut list = api::profiles(&state.http, &token).await.map_err(String::from)?;
    list.sort_by(|a, b| a["title"].as_str().unwrap_or("").cmp(b["title"].as_str().unwrap_or("")));
    *state.profiles.lock().unwrap() = list.clone();
    Ok(list)
}

#[tauri::command]
async fn server_ping(host: String, port: u16) -> R<ping::PingInfo> {
    ping::ping(&host, port).await
}

// ---------------------------------------------------------------- игра

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayOptions {
    #[serde(default)]
    ram: u64,
    #[serde(default)]
    full_screen: bool,
    #[serde(default)]
    auto_enter: bool,
}

#[tauri::command]
async fn game_play(app: tauri::AppHandle, state: State<'_, AppState>, uuid: String, options: PlayOptions) -> R<()> {
    let token = state.token()?;
    let profile = state.profiles.lock().unwrap().iter().find(|p| p["uuid"].as_str() == Some(uuid.as_str())).cloned().ok_or("Профиль не найден")?;
    state.cancel.store(false, Ordering::Relaxed);
    let updates = store::updates_dir(&state.settings.lock().unwrap());
    let dir = profile["dir"].as_str().ok_or("dir")?.to_string();
    let asset_dir = profile["assetDir"].as_str().unwrap_or("assets").to_string();
    let emit = |stage: &'static str, p: sync::Progress| {
        let _ = app.emit("sync-progress", json!({ "stage": stage, "phase": p.phase, "done": p.done, "total": p.total,
            "filesDone": p.files_done, "filesTotal": p.files_total, "file": p.file, "speed": p.speed }));
    };

    // 1. Java
    let app2 = app.clone();
    let java = java::ensure(&state.http, &move |done, total| {
        let _ = app2.emit("sync-progress", json!({ "stage": "java", "phase": "download", "done": done, "total": total, "filesDone": 0, "filesTotal": 1, "file": "java", "speed": 0 }));
    })
    .await?;

    // 2. файлы: свои библиотеки, assets, клиент
    let syncer = sync::Syncer { http: state.http.clone(), cancel: state.cancel.clone(), index: state.index.clone() };
    let index_path = paths::cxl_dir().join("index.json");
    let libs_dir = paths::cxl_dir().join("libs");
    let res: R<()> = async {
        let m = api::manifest(&state.http, &token, "@cxl").await.map_err(String::from)?;
        let rules = sync::Rules { update_all: true, ..Default::default() };
        syncer.sync(&m, &libs_dir, &rules, &|p| emit("libs", p)).await?;
        let m = api::manifest(&state.http, &token, &asset_dir).await.map_err(String::from)?;
        syncer.sync(&m, &updates.join(&asset_dir), &rules, &|p| emit("assets", p)).await?;
        let m = api::manifest(&state.http, &token, &dir).await.map_err(String::from)?;
        syncer.sync(&m, &updates.join(&dir), &sync::rules_from_profile(&profile), &|p| emit("client", p)).await?;
        Ok(())
    }
    .await;
    state.index.lock().unwrap().save(&index_path);
    res?;

    // 3. токен для игры и запуск
    let gs = api::game_session(&state.http, &token).await.map_err(String::from)?;
    let client_dir = updates.join(&dir);
    let args = launch::build_args(&launch::LaunchParams {
        profile: &profile,
        client_dir: client_dir.clone(),
        assets_dir: updates.join(&asset_dir),
        cxl_libs: libs_dir,
        ram_mb: options.ram,
        full_screen: options.full_screen,
        auto_enter: options.auto_enter,
        username: gs.username.clone(),
        uuid: gs.uuid.clone(),
        access_token: gs.access_token.clone(),
    });
    let _ = app.emit("game-log", json!({ "line": format!("{} {}", java.display(), launch::redacted(&args)) }));
    spawn_game(&app, &state, &java, &args, &client_dir)
}

fn spawn_game(app: &tauri::AppHandle, state: &State<'_, AppState>, java: &std::path::Path, args: &[String], cwd: &std::path::Path) -> R<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut cmd = tokio::process::Command::new(java);
    cmd.args(args).current_dir(cwd).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).stdin(std::process::Stdio::null());
    if let Some(home) = java.parent().and_then(|b| b.parent()) {
        cmd.env("JAVA_HOME", home);
    }
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    #[cfg(target_os = "linux")]
    cmd.env("__GL_THREADED_OPTIMIZATIONS", "0");
    let mut child = cmd.spawn().map_err(|e| format!("Не удалось запустить игру: {e}"))?;
    let log_path = paths::cxl_dir().join("logs").join("game-latest.log");
    let _ = std::fs::create_dir_all(log_path.parent().unwrap());
    let log = Arc::new(Mutex::new(std::fs::File::create(&log_path).ok()));
    for out in [child.stdout.take().map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Unpin + Send>), child.stderr.take().map(|s| Box::new(s) as Box<dyn tokio::io::AsyncRead + Unpin + Send>)].into_iter().flatten() {
        let app = app.clone();
        let log = log.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(f) = log.lock().unwrap().as_mut() {
                    use std::io::Write;
                    let _ = writeln!(f, "{line}");
                }
                let _ = app.emit("game-log", json!({ "line": line }));
            }
        });
    }
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    *state.game_kill.lock().unwrap() = Some(tx);
    let _ = app.emit("game-started", json!({ "pid": child.id() }));
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let code = tokio::select! {
            r = child.wait() => r.ok().and_then(|s| s.code()).unwrap_or(-1),
            _ = rx => { let _ = child.kill().await; let _ = child.wait().await; -1 }
        };
        if let Some(st) = app2.try_state::<AppState>() {
            *st.game_kill.lock().unwrap() = None;
        }
        let _ = app2.emit("game-exit", json!({ "code": code }));
    });
    Ok(())
}

#[tauri::command]
fn game_kill(state: State<'_, AppState>) {
    if let Some(tx) = state.game_kill.lock().unwrap().take() {
        let _ = tx.send(());
    }
}

#[tauri::command]
fn game_running(state: State<'_, AppState>) -> bool {
    state.game_kill.lock().unwrap().is_some()
}

#[tauri::command]
fn sync_cancel(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

// ---------------------------------------------------------------- окно и система

#[tauri::command]
fn window_set_size(window: tauri::WebviewWindow, width: f64, height: f64) -> R<()> {
    window.set_size(tauri::LogicalSize::new(width, height)).map_err(|e| e.to_string())?;
    let _ = window.center();
    Ok(())
}

#[tauri::command]
fn window_show(window: tauri::WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}

#[tauri::command]
fn window_minimize(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

/// закрытие лаунчера убивает запущенную игру (PrimaryStage.close старого лаунчера)
#[tauri::command]
fn window_close(app: tauri::AppHandle, state: State<'_, AppState>) {
    if let Some(tx) = state.game_kill.lock().unwrap().take() {
        let _ = tx.send(());
    }
    state.save_settings();
    state.index.lock().unwrap().save(&paths::cxl_dir().join("index.json"));
    std::thread::sleep(std::time::Duration::from_millis(150));
    app.exit(0);
}

// ---------------------------------------------------------------- папка клиентов (настройки)

#[tauri::command]
async fn updates_dir_pick(app: tauri::AppHandle) -> R<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().set_title("Сменить директорию загрузок").set_directory(paths::app_dir()).pick_folder(move |p| {
        let _ = tx.send(p.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string()));
    });
    Ok(rx.await.unwrap_or(None))
}

/// DirBridge.move: перенести уже скачанные клиенты в новую папку (rename, иначе копирование + удаление)
#[tauri::command]
async fn updates_dir_change(state: State<'_, AppState>, path: String) -> R<String> {
    let from = store::updates_dir(&state.settings.lock().unwrap());
    let to = std::path::PathBuf::from(&path);
    if from != to && from.exists() {
        let (f2, t2) = (from.clone(), to.clone());
        tokio::task::spawn_blocking(move || move_dir(&f2, &t2)).await.map_err(|e| e.to_string())??;
    }
    state.settings.lock().unwrap()["updatesDir"] = json!(path);
    state.save_settings();
    Ok(path)
}

fn move_dir(from: &std::path::Path, to: &std::path::Path) -> R<()> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for e in std::fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
        let dst = to.join(e.file_name());
        if std::fs::rename(e.path(), &dst).is_err() {
            copy_tree(&e.path(), &dst)?;
            let _ = if e.path().is_dir() { std::fs::remove_dir_all(e.path()) } else { std::fs::remove_file(e.path()) };
        }
    }
    let _ = std::fs::remove_dir(from);
    Ok(())
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) -> R<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
        for e in std::fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
            copy_tree(&e.path(), &to.join(e.file_name()))?;
        }
    } else {
        std::fs::copy(from, to).map_err(|e| format!("{}: {e}", from.display()))?;
    }
    Ok(())
}

#[tauri::command]
fn open_path(path: String) -> R<()> {
    open::that_detached(path).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- самообновление

#[tauri::command]
async fn self_update_check(state: State<'_, AppState>) -> R<Option<selfupdate::Release>> {
    Ok(selfupdate::check(&state.http).await)
}

#[tauri::command]
async fn self_update_apply(app: tauri::AppHandle, state: State<'_, AppState>, release: selfupdate::Release) -> R<()> {
    let app2 = app.clone();
    selfupdate::apply(&state.http, &release, &move |done, total| {
        let _ = app2.emit("selfupdate-progress", json!({ "done": done, "total": total }));
    })
    .await?;
    state.save_settings();
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn open_url(url: String) -> R<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("bad url".into());
    }
    open::that_detached(url).map_err(|e| e.to_string())
}

pub fn run() {
    if selfupdate::on_startup() {
        return; // обновление со старого лаунчера поставлено, запущен установленный exe
    }
    let store = Store::open();
    let settings = store.load_settings();
    let index = sync::Index::load(&paths::cxl_dir().join("index.json"));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(AppState {
            http: api::client(),
            store,
            settings: Mutex::new(settings),
            session: Mutex::new(None),
            profiles: Mutex::new(Vec::new()),
            cancel: Arc::new(AtomicBool::new(false)),
            index: Arc::new(Mutex::new(index)),
            game_kill: Mutex::new(None),
        })
        .setup(|app| {
            // страховка: если интерфейс не показал окно (ошибка JS и т.п.) - показать через 3 с
            if let Some(w) = app.get_webview_window("main") {
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    if !w.is_visible().unwrap_or(true) {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_init, settings_save, auth_login, auth_restore, auth_remember, auth_logout, accounts_list,
            profiles_list, server_ping, game_play, game_kill, game_running, sync_cancel,
            window_set_size, window_show, window_minimize, window_close, open_url,
            updates_dir_pick, updates_dir_change, open_path, self_update_check, self_update_apply
        ])
        .run(tauri::generate_context!())
        .expect("error while running CubeXCraft launcher");
}
