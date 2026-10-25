// HTTP-клиент к своему API на сайте (website/launcher-api.js) вместо WebSocket LaunchServer.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SITE: &str = "https://www.cubexcraft.site";
const API: &str = "https://www.cubexcraft.site/api/launcher/v1";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub uuid: String,
    pub username: String,
    #[serde(default)]
    pub skin: Option<String>,
    #[serde(default)]
    pub cloak: Option<String>,
}

#[derive(Deserialize)]
struct LoginResp {
    #[serde(rename = "accessToken")]
    access_token: String,
    user: User,
}

#[derive(Deserialize)]
struct MeResp {
    user: User,
}

#[derive(Deserialize)]
struct ProfilesResp {
    profiles: Vec<Value>,
}

#[derive(Deserialize, Clone)]
pub struct GameSession {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub uuid: String,
    pub username: String,
}

/// ошибка для интерфейса: "auth.invalidtoken" (сессия умерла), иначе - готовый текст с сервера
#[derive(Debug, Clone)]
pub struct ApiError(pub String);

impl From<ApiError> for String {
    fn from(e: ApiError) -> String {
        e.0
    }
}

async fn fail(r: reqwest::Response) -> ApiError {
    let status = r.status();
    let body: Value = r.json().await.unwrap_or(Value::Null);
    if status == reqwest::StatusCode::UNAUTHORIZED && body.get("error").and_then(|e| e.as_str()) == Some("auth") {
        return ApiError("auth.invalidtoken".into());
    }
    ApiError(
        body.get("message")
            .and_then(|m| m.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("Ошибка сервера ({})", status.as_u16())),
    )
}

fn net(e: reqwest::Error) -> ApiError {
    ApiError(format!("Нет соединения с сервером: {}", e.without_url()))
}

/// запрос подтверждения входа в Telegram (у игрока привязан бот, сайт: telegram-2fa.js)
#[derive(Deserialize, Clone, Debug)]
pub struct TwoFa {
    pub challenge: String,
    #[serde(rename = "expiresIn", default)]
    pub expires_in: u64,
    #[serde(default)]
    pub bot: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LoginAny {
    Done(LoginResp),
    TwoFa { twofa: TwoFa },
}

pub enum LoginOutcome {
    Done(String, User),
    TwoFa(TwoFa),
}

/// twofa: "poll" - лаунчер сам ждёт подтверждения в Telegram (своё окно ожидания);
/// "skip" - перенос паролей старого лаунчера: если включена 2FA, сразу отказ, без запроса в бот
pub async fn login(http: &reqwest::Client, login: &str, password: &str, device: &str, twofa: &str) -> Result<LoginOutcome, ApiError> {
    let r = http
        .post(format!("{API}/auth/login"))
        .json(&serde_json::json!({ "login": login, "password": password, "device": device, "twofa": twofa }))
        .send()
        .await
        .map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    match r.json::<LoginAny>().await.map_err(net)? {
        LoginAny::Done(v) => Ok(LoginOutcome::Done(v.access_token, v.user)),
        LoginAny::TwoFa { twofa } => Ok(LoginOutcome::TwoFa(twofa)),
    }
}

pub enum TwoFaPoll {
    Pending,
    Done(String, User),
}

/// решение по запросу: сервер держит ответ до ~20 с, пока игрок не нажмёт кнопку в боте
pub async fn twofa_poll(http: &reqwest::Client, challenge: &str) -> Result<TwoFaPoll, ApiError> {
    let r = http
        .post(format!("{API}/auth/2fa"))
        .json(&serde_json::json!({ "challenge": challenge }))
        .timeout(std::time::Duration::from_secs(40))
        .send()
        .await
        .map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    let v: Value = r.json().await.map_err(net)?;
    if v.get("accessToken").is_some() {
        let d: LoginResp = serde_json::from_value(v).map_err(|e| ApiError(format!("Плохой ответ сервера: {e}")))?;
        return Ok(TwoFaPoll::Done(d.access_token, d.user));
    }
    Ok(TwoFaPoll::Pending)
}

pub async fn twofa_cancel(http: &reqwest::Client, challenge: &str) {
    let _ = http
        .post(format!("{API}/auth/2fa/cancel"))
        .json(&serde_json::json!({ "challenge": challenge }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
}

pub async fn me(http: &reqwest::Client, token: &str) -> Result<User, ApiError> {
    let r = http.get(format!("{API}/me")).bearer_auth(token).send().await.map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    Ok(r.json::<MeResp>().await.map_err(net)?.user)
}

pub async fn logout(http: &reqwest::Client, token: &str) {
    let _ = http.post(format!("{API}/auth/logout")).bearer_auth(token).send().await;
}

pub async fn profiles(http: &reqwest::Client, token: &str) -> Result<Vec<Value>, ApiError> {
    let r = http.get(format!("{API}/profiles")).bearer_auth(token).send().await.map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    Ok(r.json::<ProfilesResp>().await.map_err(net)?.profiles)
}

pub async fn game_session(http: &reqwest::Client, token: &str) -> Result<GameSession, ApiError> {
    let r = http.post(format!("{API}/game/session")).bearer_auth(token).send().await.map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    r.json().await.map_err(net)
}

/// манифест файлов: { base, files: { "path": [size, "sha1"] } }
#[derive(Deserialize, Clone, Debug)]
pub struct Manifest {
    pub base: String,
    pub files: std::collections::BTreeMap<String, (u64, String)>,
}

pub async fn manifest(http: &reqwest::Client, token: &str, dir: &str) -> Result<Manifest, ApiError> {
    let r = http
        .get(format!("{API}/files/{}", urlencode(dir)))
        .bearer_auth(token)
        .send()
        .await
        .map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    r.json().await.map_err(|e| ApiError(format!("Плохой ответ сервера: {e}")))
}

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~/".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// друзья/статус (launcher-friends.js на сайте): GET без тела или POST с телом; ответ - снимок
/// { friends, requests, notifications } (+ result/username у действий), 204 - Value::Null
pub async fn social(http: &reqwest::Client, token: &str, path: &str, body: Option<&Value>) -> Result<Value, ApiError> {
    let url = format!("{API}/{path}");
    let req = match body {
        Some(b) => http.post(url).json(b),
        None => http.get(url),
    };
    let r = req.bearer_auth(token).timeout(std::time::Duration::from_secs(15)).send().await.map_err(net)?;
    if r.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(Value::Null);
    }
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    r.json().await.map_err(net)
}

/// Новости сайта для правой колонки (без входа, website/launcher-api.js /news) и данные кнопки «Голосовать».
/// Обложки качаются здесь, тем же клиентом (через ретранслятор, если он включён), и уходят в интерфейс как data:-адреса:
/// сам WebView ходил бы к сайту напрямую, а из РФ такие соединения режут после ~16 КБ. Обложка, которая не скачалась, — null.
pub async fn news(http: &reqwest::Client) -> Result<Value, ApiError> {
    use base64::Engine;
    const MAX_COVER: usize = 2 << 20;
    let r = http.get(format!("{API}/news")).timeout(std::time::Duration::from_secs(20)).send().await.map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    let mut v: Value = r.json().await.map_err(net)?;
    if let Some(list) = v.get_mut("news").and_then(|n| n.as_array_mut()) {
        for item in list.iter_mut() {
            let Some(url) = item.get("cover").and_then(|c| c.as_str()).map(String::from) else { continue };
            let data = async {
                if !url.starts_with("https://www.cubexcraft.site/") {
                    return None;
                }
                let r = http.get(&url).timeout(std::time::Duration::from_secs(20)).send().await.ok()?;
                if !r.status().is_success() {
                    return None;
                }
                let mime = r.headers().get(reqwest::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("image/jpeg").to_string();
                if !mime.starts_with("image/") {
                    return None;
                }
                let bytes = r.bytes().await.ok()?;
                (bytes.len() <= MAX_COVER).then(|| format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(&bytes)))
            }
            .await;
            item["cover"] = data.map(Value::String).unwrap_or(Value::Null);
        }
    }
    Ok(v)
}
