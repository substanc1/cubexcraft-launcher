// Сборка командной строки игры - как GravitLauncher ClientLauncherProcess.start + ClientParams.addModernClientArgs
// (режим SYSTEM_ARGS, moduleConf, HIDE_SYSTEM_ARGS_CLASSPATH), но без промежуточного ClientLauncherEntryPoint:
// сразу mainClass профиля (BootstrapLauncher), обычный authlib + -Dminecraft.api.* (их добавляет сервер в jvmArgs).
use crate::paths;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct LaunchParams<'a> {
    pub profile: &'a Value,
    pub client_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub cxl_libs: PathBuf,
    pub ram_mb: u64,
    pub full_screen: bool,
    pub auto_enter: bool,
    /// размер окна игры (game_window_size) - одинаковый для всех профилей
    pub window: (u32, u32),
    pub username: String,
    pub uuid: String,
    pub access_token: String,
}

fn strs(v: &Value, k: &str) -> Vec<String> {
    v.get(k).and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect()).unwrap_or_default()
}

fn sep() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

fn subst(s: &str, version: &str) -> String {
    s.replace("${version_name}", version).replace("${library_directory}", "libraries").replace("${classpath_separator}", sep())
}

/// "@cxl/libs/x.jar" -> абсолютный путь к своим библиотекам лаунчера, остальное - относительно папки клиента
fn resolve(p: &str, cxl_libs: &Path) -> String {
    if let Some(rest) = p.strip_prefix("@cxl/libs/") {
        cxl_libs.join(rest).to_string_lossy().to_string()
    } else {
        p.to_string()
    }
}

/// os-триггер optional-действий профиля (MacOSArgs: -XstartOnFirstThread только на macOS)
fn optional_jvm_args(profile: &Value) -> Vec<String> {
    let mut out = Vec::new();
    let my_os = if cfg!(windows) { "MUSTDIE" } else if cfg!(target_os = "macos") { "MACOSX" } else { "LINUX" };
    for opt in profile.get("updateOptional").and_then(|v| v.as_array()).into_iter().flatten() {
        let triggers = opt.get("triggersList").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let os_ok = triggers.iter().all(|t| {
            if t.get("type").and_then(|v| v.as_str()) != Some("os") {
                return true;
            }
            let want = t.get("os").and_then(|v| v.as_str()).unwrap_or("");
            let inverted = t.get("inverted").and_then(|v| v.as_bool()).unwrap_or(false);
            (want == my_os) != inverted
        });
        if !os_ok || triggers.is_empty() {
            continue;
        }
        for a in opt.get("actions").and_then(|v| v.as_array()).into_iter().flatten() {
            if a.get("type").and_then(|v| v.as_str()) == Some("jvmArgs") {
                out.extend(strs(a, "args"));
            }
        }
    }
    out
}

pub fn build_args(p: &LaunchParams) -> Vec<String> {
    let prof = p.profile;
    let version = prof.get("version").and_then(|v| v.as_str()).unwrap_or("1.21.1").to_string();
    let mut args: Vec<String> = Vec::new();
    for a in strs(prof, "jvmArgs") {
        args.push(subst(&a, &version));
    }
    args.extend(optional_jvm_args(prof));
    if p.ram_mb > 0 {
        args.push(format!("-Xmx{}M", p.ram_mb));
    }
    let natives = p.client_dir.join("natives").join(paths::natives_os()).join(paths::game_arch());
    args.push(format!("-Djava.library.path={}", natives.to_string_lossy()));

    let mc = prof.get("moduleConf").cloned().unwrap_or(Value::Null);
    let module_path = strs(&mc, "modulePath");
    if !module_path.is_empty() {
        args.push("-p".into());
        args.push(module_path.iter().map(|m| resolve(m, &p.cxl_libs)).collect::<Vec<_>>().join(sep()));
    }
    let modules = strs(&mc, "modules");
    if !modules.is_empty() {
        args.push("--add-modules".into());
        args.push(modules.join(","));
    }
    for (flag, key) in [("--add-exports", "exports"), ("--add-opens", "opens"), ("--add-reads", "reads")] {
        if let Some(m) = mc.get(key).and_then(|v| v.as_object()) {
            for (k, v) in m {
                if let Some(v) = v.as_str() {
                    args.push(flag.into());
                    args.push(format!("{k}={v}"));
                }
            }
        }
    }
    // classpath: classPath профиля без модулей + altClassPath
    let mut cp: Vec<String> = strs(prof, "classPath")
        .into_iter()
        .filter(|c| !module_path.contains(c))
        .map(|c| resolve(&c, &p.cxl_libs))
        .collect();
    cp.extend(strs(prof, "altClassPath").into_iter().map(|c| resolve(&c, &p.cxl_libs)));
    // Gravit клал classPath и altClassPath в разные места (HIDE_SYSTEM_ARGS_CLASSPATH), у нас всё в одном -cp:
    // jar из обоих списков (cxc-mcversion.jar) дал бы «Duplicate key» в securejarhandler BootstrapLauncher
    let mut seen = std::collections::HashSet::new();
    cp.retain(|c| seen.insert(c.to_lowercase()));
    args.push("-cp".into());
    args.push(cp.join(sep()));
    args.push(prof.get("mainClass").and_then(|v| v.as_str()).unwrap_or("cpw.mods.bootstraplauncher.BootstrapLauncher").to_string());

    // ClientParams.addModernClientArgs
    let client = p.client_dir.to_string_lossy().to_string();
    args.extend([
        "--username".into(), p.username.clone(),
        "--uuid".into(), p.uuid.replace('-', ""),
        "--accessToken".into(), p.access_token.clone(),
        "--userType".into(), "mojang".into(),
        "--userProperties".into(), "{}".into(),
        "--assetIndex".into(), prof.get("assetIndex").and_then(|v| v.as_str()).unwrap_or(&version).to_string(),
        "--version".into(), version.clone(),
        "--gameDir".into(), client.clone(),
        "--assetsDir".into(), p.assets_dir.to_string_lossy().to_string(),
        "--resourcePackDir".into(), p.client_dir.join("resourcepacks").to_string_lossy().to_string(),
        "--versionType".into(), "CubeXCraft".into(),
    ]);
    if p.auto_enter {
        if let Some(s) = prof.get("servers").and_then(|v| v.as_array()).and_then(|a| a.first()) {
            let host = s.get("serverAddress").and_then(|v| v.as_str()).unwrap_or("");
            let port = s.get("serverPort").and_then(|v| v.as_u64()).unwrap_or(25565);
            args.push("--quickPlayMultiplayer".into());
            args.push(format!("{host}:{port}"));
        }
    }
    if p.full_screen {
        args.push("--fullscreen".into());
        args.push("true".into());
    }
    // размер окна задаёт лаунчер под экран игрока: в профилях было --width 1708 --height 960 в пикселях -
    // на 1080p это почти весь экран, на ноутбуке и на Mac (там это точки) окно вылезало за экран
    let mut client_args = strs(prof, "clientArgs");
    let mut i = 0;
    while i < client_args.len() {
        if client_args[i] == "--width" || client_args[i] == "--height" {
            client_args.drain(i..(i + 2).min(client_args.len()));
        } else {
            i += 1;
        }
    }
    args.extend(client_args);
    args.extend(["--width".into(), p.window.0.to_string(), "--height".into(), p.window.1.to_string()]);
    args
}

/// Полноэкранный режим определяет только настройка профиля в лаунчере: Minecraft запоминает F11 в options.txt,
/// и следующая игра открывалась то в окне, то на весь экран
pub fn sync_fullscreen_option(client_dir: &std::path::Path, full_screen: bool) {
    let path = client_dir.join("options.txt");
    let Ok(text) = std::fs::read_to_string(&path) else { return };
    let want = format!("fullscreen:{full_screen}");
    let mut changed = false;
    let lines: Vec<String> = text
        .lines()
        .map(|l| {
            if l.starts_with("fullscreen:") && l != want {
                changed = true;
                want.clone()
            } else {
                l.to_string()
            }
        })
        .collect();
    if changed {
        let _ = std::fs::write(&path, lines.join("
") + "
");
    }
}

/// строка для журнала: токен не показываем
pub fn redacted(args: &[String]) -> String {
    let mut out = Vec::with_capacity(args.len());
    let mut hide = false;
    for a in args {
        out.push(if hide { "***".to_string() } else { a.clone() });
        hide = a == "--accessToken";
    }
    out.join(" ")
}
