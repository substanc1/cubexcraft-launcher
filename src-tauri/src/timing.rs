// Журнал времени запуска: cxl/logs/launcher.log - сколько занял каждый шаг от старта лаунчера до игры
// (по нему видно, что тормозит у конкретного игрока). Перезаписывается при каждом запуске лаунчера.
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static START: OnceLock<Instant> = OnceLock::new();
static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init() {
    START.get_or_init(Instant::now);
    let path = crate::paths::cxl_dir().join("logs").join("launcher.log");
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    *FILE.lock().unwrap() = std::fs::File::create(path).ok();
    mark(&format!("лаунчер {} запущен", env!("CARGO_PKG_VERSION")));
}

pub fn mark(what: &str) {
    let t = START.get_or_init(Instant::now).elapsed().as_secs_f64();
    if let Some(f) = FILE.lock().unwrap().as_mut() {
        let _ = writeln!(f, "[{t:7.2}s] {what}");
    }
}
