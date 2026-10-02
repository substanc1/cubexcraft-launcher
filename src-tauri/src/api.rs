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

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("CubeXCraft-Launcher/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60))
        .gzip(true)
        .build()
        .expect("http client")
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

pub async fn login(http: &reqwest::Client, login: &str, password: &str, device: &str) -> Result<(String, User), ApiError> {
    let r = http
        .post(format!("{API}/auth/login"))
        .json(&serde_json::json!({ "login": login, "password": password, "device": device }))
        .send()
        .await
        .map_err(net)?;
    if !r.status().is_success() {
        return Err(fail(r).await);
    }
    let v: LoginResp = r.json().await.map_err(net)?;
    Ok((v.access_token, v.user))
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
