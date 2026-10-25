// Маршрут к сайту и игре для игроков из РФ (Jira SCRUM-126, 18.10). У части провайдеров ТСПУ режет соединения к нашему
// зарубежному серверу: ~16 КБ данных и дальше тишина (лаунчер висит на 0 МБ/с, игра — на входе). Лаунчер при старте
// качает файл 64 КБ напрямую; если он встал, а через ретранслятор в Москве проходит — всё сетевое (сайт, файлы, вход
// и сервер игры) идёт через ретранслятор. Имя сайта и сертификат те же: ретранслятор только пересылает байты, TLS не
// вскрывает. Выбор запоминается (cxl/route.json), чтобы со второго запуска не ждать проверки.
use crate::paths;
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

pub const RELAY_IP: Ipv4Addr = Ipv4Addr::new(89, 125, 214, 40);
/// имена, которые в режиме ретранслятора смотрят на него: сайт и файлы (www, голый домен), сервер игры (play)
pub const HOSTS: [&str; 3] = ["www.cubexcraft.site", "cubexcraft.site", "play.cubexcraft.site"];
const PROBE: &str = "https://www.cubexcraft.site/cxl/probe-64k.bin";
const PROBE_LEN: usize = 65536;

static RELAY: AtomicBool = AtomicBool::new(false);
static DECIDED: AtomicBool = AtomicBool::new(false);
static CLIENT: OnceLock<RwLock<reqwest::Client>> = OnceLock::new();

fn ready_notify() -> &'static tokio::sync::Notify {
    static N: OnceLock<tokio::sync::Notify> = OnceLock::new();
    N.get_or_init(tokio::sync::Notify::new)
}

pub fn relay() -> bool {
    RELAY.load(Ordering::Relaxed)
}

fn build(relay: bool) -> reqwest::Client {
    let mut b = reqwest::Client::builder()
        .user_agent(concat!("CubeXCraft-Launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .gzip(true);
    if relay {
        // порт из адреса reqwest не берёт — идёт обычный 443 из https://
        for h in HOSTS {
            b = b.resolve(h, SocketAddr::new(IpAddr::V4(RELAY_IP), 443));
        }
    }
    b.build().expect("http client")
}

/// HTTP-клиент текущего маршрута (дёшево клонируется)
pub fn http() -> reqwest::Client {
    CLIENT.get_or_init(|| RwLock::new(build(cached()))).read().unwrap().clone()
}

fn set(relay: bool) {
    RELAY.store(relay, Ordering::Relaxed);
    let lock = CLIENT.get_or_init(|| RwLock::new(build(relay)));
    *lock.write().unwrap() = build(relay);
}

fn route_file() -> std::path::PathBuf {
    paths::cxl_dir().join("route.json")
}

fn cached() -> bool {
    std::fs::read_to_string(route_file())
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("relay").and_then(|b| b.as_bool()))
        .unwrap_or(false)
}

fn save(relay: bool) {
    let _ = std::fs::write(route_file(), json!({ "relay": relay }).to_string());
}

/// до старта: маршрут прошлого запуска (у кого режут — сразу через ретранслятор, без ожидания проверки)
pub fn init() {
    let c = cached();
    RELAY.store(c, Ordering::Relaxed);
    let _ = CLIENT.set(RwLock::new(build(c)));
}

/// файл 64 КБ целиком за 8 с: у кого режут, прямой путь встаёт на ~16 КБ
async fn probe(relay: bool) -> bool {
    let c = build(relay);
    let fut = async {
        let r = c.get(PROBE).timeout(Duration::from_secs(8)).send().await.ok()?;
        if !r.status().is_success() {
            return None;
        }
        let b = r.bytes().await.ok()?;
        Some(b.len() == PROBE_LEN)
    };
    tokio::time::timeout(Duration::from_secs(9), fut).await.ok().flatten().unwrap_or(false)
}

/// проверка пути при старте (фоном); всё сетевое ждёт её через ready()
pub async fn decide() {
    let was = cached();
    // проверка у тех, у кого ничего не режут: CXL_FORCE_RELAY=1 — сразу через ретранслятор (выбор не запоминается)
    if std::env::var("CXL_FORCE_RELAY").map(|v| v == "1").unwrap_or(false) {
        set(true);
        crate::timing::mark("маршрут: через ретранслятор РФ (CXL_FORCE_RELAY)");
        DECIDED.store(true, Ordering::Relaxed);
        ready_notify().notify_waiters();
        return;
    }
    let relay = if was {
        if probe(true).await {
            // прямой снова работает — со следующего запуска прямо (в этом запуске маршрут не меняем)
            tauri::async_runtime::spawn(async {
                if probe(false).await {
                    save(false);
                }
            });
            true
        } else {
            false // ретранслятор не отвечает — прямо
        }
    } else if probe(false).await {
        false
    } else {
        probe(true).await
    };
    set(relay);
    if relay != was {
        save(relay);
    }
    crate::timing::mark(if relay { "маршрут: через ретранслятор РФ (прямой путь режут)" } else { "маршрут: прямой" });
    DECIDED.store(true, Ordering::Relaxed);
    ready_notify().notify_waiters();
}

/// дождаться проверки пути (не дольше 20 с: без ответа идём тем маршрутом, что есть)
pub async fn ready() {
    let n = ready_notify().notified();
    if DECIDED.load(Ordering::Relaxed) {
        return;
    }
    let _ = tokio::time::timeout(Duration::from_secs(20), n).await;
}

/// хост для своего TCP (пинг сервера в лаунчере): наш — на ретранслятор
pub fn tcp_host(host: &str) -> String {
    if relay() && HOSTS.contains(&host.to_ascii_lowercase().as_str()) {
        RELAY_IP.to_string()
    } else {
        host.to_string()
    }
}

/// ссылка на наш сайт для браузера игрока: через ретранслятор — на ru.cubexcraft.site (браузер обход не знает, а ru смотрит
/// на ретранслятор и отдаёт тот же сайт); прочие ссылки и прямой маршрут — без изменений
pub fn site_url(url: &str) -> String {
    if !relay() {
        return url.to_string();
    }
    for h in ["https://www.cubexcraft.site", "https://cubexcraft.site", "http://www.cubexcraft.site", "http://cubexcraft.site"] {
        if let Some(rest) = url.strip_prefix(h) {
            if rest.is_empty() || rest.starts_with('/') || rest.starts_with('?') || rest.starts_with('#') {
                return format!("https://ru.cubexcraft.site{rest}");
            }
        }
    }
    url.to_string()
}

/// свойство для резолвера в игре (cxc-relay.jar): наши имена -> ретранслятор, остальные как обычно
pub fn jvm_hosts_prop() -> String {
    let list: Vec<String> = HOSTS.iter().map(|h| format!("{h}={RELAY_IP}")).collect();
    format!("-Dcubexcraft.relay.hosts={}", list.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_url_through_relay() {
        RELAY.store(false, Ordering::Relaxed);
        assert_eq!(site_url("https://cubexcraft.site/register"), "https://cubexcraft.site/register");
        RELAY.store(true, Ordering::Relaxed);
        assert_eq!(site_url("https://cubexcraft.site/register"), "https://ru.cubexcraft.site/register");
        assert_eq!(site_url("https://www.cubexcraft.site/forgot-password"), "https://ru.cubexcraft.site/forgot-password");
        assert_eq!(site_url("https://www.cubexcraft.site"), "https://ru.cubexcraft.site");
        assert_eq!(site_url("https://www.cubexcraft.site?x=1"), "https://ru.cubexcraft.site?x=1");
        assert_eq!(site_url("https://discord.gg/abc"), "https://discord.gg/abc");
        assert_eq!(site_url("https://cubexcraft.site.evil.com/x"), "https://cubexcraft.site.evil.com/x");
        assert_eq!(tcp_host("play.cubexcraft.site"), "89.125.214.40");
        assert_eq!(tcp_host("example.org"), "example.org");
        RELAY.store(false, Ordering::Relaxed);
    }
}
