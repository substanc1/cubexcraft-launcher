// Журналы лаунчера: cxl/logs
//  * launcher.log, game-latest.log, game-window.log - последний запуск, перезаписываются каждый раз;
//  * crashes/ - сохраняются при ошибке: game-<время>.log (игра вылетела: её журнал + отчёт о краше Minecraft +
//    launcher.log), launcher-<время>.log (ошибки лаунчера за один его запуск - в одном файле). Хранятся последние
//    KEEP файлов, остальные удаляются.
// Заодно у клиента игры чистятся старые logs/*.log.gz и crash-reports (Minecraft их сам не удаляет никогда).
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

const KEEP: usize = 10;
/// ошибка лаунчера пишется в файл этого запуска лаунчера (создаётся при первой ошибке)
static LAUNCHER_ERRORS: Mutex<Option<(PathBuf, u32)>> = Mutex::new(None);

pub fn dir() -> PathBuf {
    crate::paths::cxl_dir().join("logs")
}

fn crashes() -> PathBuf {
    dir().join("crashes")
}

/// «2026-10-02_13-50-12» (UTC) - для имён файлов и заголовков
fn stamp(t: SystemTime) -> String {
    let s = t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (s.div_euclid(86400), s.rem_euclid(86400));
    // дни от 1970-01-01 -> дата (алгоритм civil_from_days)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}_{:02}-{:02}-{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

fn header(f: &mut std::fs::File, what: &str) {
    let _ = writeln!(
        f,
        "CubeXCraft launcher {} · {} · {}\n{} (время UTC)\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        crate::paths::game_arch(),
        what
    );
}

fn append_file(f: &mut std::fs::File, title: &str, path: &Path) {
    if let Ok(text) = std::fs::read(path) {
        let _ = writeln!(f, "\n===== {title} ({}) =====", path.display());
        let _ = f.write_all(&text);
        let _ = writeln!(f);
    }
}

/// Игра вылетела (код != 0, не остановлена кнопкой): копия её журнала, свежий отчёт о краше Minecraft (если
/// появился после старта) и журнал шагов лаунчера. Возвращает имя сохранённого файла.
pub fn save_game_crash(code: i32, client_dir: &Path, started: SystemTime) -> Option<String> {
    let dir = crashes();
    std::fs::create_dir_all(&dir).ok()?;
    let name = format!("game-{}.log", stamp(SystemTime::now()));
    let mut f = std::fs::File::create(dir.join(&name)).ok()?;
    header(&mut f, &format!("Игра завершилась с кодом {code} · {}", stamp(SystemTime::now())));
    // отчёт Minecraft (crash-reports/crash-...-client.txt), созданный за этот запуск
    let report = std::fs::read_dir(client_dir.join("crash-reports")).ok().and_then(|rd| {
        rd.flatten()
            .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
            .filter(|(t, _)| *t >= started)
            .max_by_key(|(t, _)| *t)
            .map(|(_, p)| p)
    });
    if let Some(p) = report {
        append_file(&mut f, "отчёт о краше Minecraft", &p);
    }
    append_file(&mut f, "журнал игры", &dir.parent().unwrap().join("game-latest.log"));
    append_file(&mut f, "журнал лаунчера", &dir.parent().unwrap().join("launcher.log"));
    drop(f);
    prune(&dir, KEEP);
    Some(name)
}

/// Ошибка, показанная игроку (вход, загрузка файлов, запуск...): в файл этого запуска лаунчера.
pub fn save_launcher_error(msg: &str) {
    let mut cur = LAUNCHER_ERRORS.lock().unwrap();
    let path = match cur.as_mut() {
        // не больше 500 строк за запуск (если сеть пропала и всё подряд падает - файл не растёт бесконечно)
        Some((_, n)) if *n >= 500 => return,
        Some((p, n)) => {
            *n += 1;
            p.clone()
        }
        None => {
            let dir = crashes();
            if std::fs::create_dir_all(&dir).is_err() {
                return;
            }
            let p = dir.join(format!("launcher-{}.log", stamp(SystemTime::now())));
            if let Ok(mut f) = std::fs::File::create(&p) {
                header(&mut f, "Ошибки лаунчера за этот запуск");
            }
            prune(&dir, KEEP);
            *cur = Some((p.clone(), 1));
            p
        }
    };
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&path) {
        let _ = writeln!(f, "[{}] {}", stamp(SystemTime::now()), msg.trim());
    }
}

/// Оставить в папке keep самых свежих файлов, остальные удалить.
fn prune(dir: &Path, keep: usize) {
    prune_where(dir, keep, |_| true);
}

fn prune_where(dir: &Path, keep: usize, want: impl Fn(&str) -> bool) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<(SystemTime, PathBuf)> = rd
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|e| want(&e.file_name().to_string_lossy()))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    if files.len() <= keep {
        return;
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, p) in files.into_iter().skip(keep) {
        let _ = std::fs::remove_file(p);
    }
}

/// Перед запуском игры: у клиента старые журналы Minecraft (logs/*.log.gz) и отчёты о крашах - последние KEEP.
pub fn prune_client(client_dir: &Path) {
    prune_where(&client_dir.join("logs"), KEEP, |n| n.ends_with(".log.gz"));
    prune_where(&client_dir.join("crash-reports"), KEEP, |n| n.ends_with(".txt"));
}
