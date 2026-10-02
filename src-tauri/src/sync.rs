// Синхронизация файлов клиента с сервером. Правила - как у GravitLauncher (HashedDir.diff + FileNameMatcher):
//  * файла нет локально -> скачать (любой путь);
//  * файл отличается -> перезаписать, только если путь под update/updateVerify профиля (и не в updateExclusions);
//  * лишний локальный файл -> удалить, только если путь под update/updateVerify (миры, скриншоты, логи,
//    options.txt и т.п. не трогаются никогда).
// Отличие от старого лаунчера - скорость: тот при КАЖДОМ запуске пересчитывал sha1 всех файлов, включая
// 1.1 ГБ assets, в один поток. Здесь: updateVerify (моды/библиотеки/natives) хэшируются параллельно на всех
// ядрах, остальное - по кэшу (размер+время изменения), скачивание в несколько потоков.
use crate::api::Manifest;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

#[derive(Clone, Default)]
pub struct Rules {
    pub update: Vec<String>,
    pub verify: Vec<String>,
    pub exclusions: Vec<String>,
    /// удалять ли лишние файлы под update/verify (для клиента - да, для assets/своих библиотек - нет)
    pub delete_extra: bool,
    /// всё считается «обновляемым» (assets, свои библиотеки лаунчера)
    pub update_all: bool,
}

impl Rules {
    fn any(list: &[String], p: &str) -> bool {
        list.iter().any(|e| p.starts_with(e.as_str()))
    }
    pub fn should_update(&self, p: &str) -> bool {
        if Self::any(&self.exclusions, p) {
            return false;
        }
        self.update_all || Self::any(&self.update, p) || Self::any(&self.verify, p)
    }
    pub fn should_verify(&self, p: &str) -> bool {
        Self::any(&self.verify, p) && !Self::any(&self.exclusions, p)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Progress {
    pub phase: &'static str, // check | download | done
    pub done: u64,
    pub total: u64,
    pub files_done: u64,
    pub files_total: u64,
    pub file: String,
    pub speed: f64, // байт/с
}

/// кэш sha1 по (размер, mtime): cxl/index.json
#[derive(Serialize, Deserialize, Default)]
pub struct Index {
    pub files: HashMap<String, (u64, u128, String)>,
}

impl Index {
    pub fn load(path: &Path) -> Index {
        std::fs::read(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }
    pub fn save(&self, path: &Path) {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, serde_json::to_vec(self).unwrap_or_default()).is_ok() {
            let _ = std::fs::rename(tmp, path);
        }
    }
}

fn stat(p: &Path) -> Option<(u64, u128)> {
    let m = std::fs::metadata(p).ok()?;
    if !m.is_file() {
        return None;
    }
    let mt = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    Some((m.len(), mt))
}

pub fn sha1_file(p: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(p)?;
    let mut h = Sha1::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

struct Job {
    rel: String,
    url: String,
    dest: PathBuf,
    size: u64,
    sha1: String,
}

pub struct Syncer {
    pub http: reqwest::Client,
    pub cancel: Arc<AtomicBool>,
    pub index: Arc<Mutex<Index>>,
}

impl Syncer {
    /// Привести каталог root к манифесту. emit - прогресс (вызывается не чаще ~10 раз/с).
    pub async fn sync(&self, m: &Manifest, root: &Path, rules: &Rules, emit: &(dyn Fn(Progress) + Send + Sync)) -> Result<(), String> {
        std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
        // 1. проверка: что качать
        let entries: Vec<(String, u64, String)> = m.files.iter().map(|(k, (s, h))| (k.clone(), *s, h.clone())).collect();
        let total_check = entries.len() as u64;
        let checked = Arc::new(AtomicU64::new(0));
        let need: Arc<Mutex<Vec<Job>>> = Arc::new(Mutex::new(Vec::new()));
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 16);
        let chunks: Vec<Vec<(String, u64, String)>> = {
            let mut c: Vec<Vec<_>> = (0..threads).map(|_| Vec::new()).collect();
            for (i, e) in entries.into_iter().enumerate() {
                c[i % threads].push(e);
            }
            c
        };
        let mut handles = Vec::new();
        for chunk in chunks {
            let root = root.to_path_buf();
            let rules = rules.clone();
            let base = m.base.clone();
            let index = self.index.clone();
            let need = need.clone();
            let checked = checked.clone();
            let cancel = self.cancel.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                for (rel, size, sha1) in chunk {
                    if cancel.load(Ordering::Relaxed) {
                        return;
                    }
                    let dest = root.join(&rel);
                    let key = dest.to_string_lossy().to_string();
                    let ok = match stat(&dest) {
                        None => false,
                        Some(_) if !rules.should_update(&rel) => true, // есть, и правилами не обновляется
                        Some((s, mt)) if s != size => {
                            let _ = mt;
                            false
                        }
                        Some((s, mt)) => {
                            let cached = index.lock().unwrap().files.get(&key).cloned();
                            let use_cache = !rules.should_verify(&rel);
                            match cached {
                                Some((cs, cm, ch)) if use_cache && cs == s && cm == mt => ch == sha1,
                                _ => match sha1_file(&dest) {
                                    Ok(h) => {
                                        index.lock().unwrap().files.insert(key.clone(), (s, mt, h.clone()));
                                        h == sha1
                                    }
                                    Err(_) => false,
                                },
                            }
                        }
                    };
                    if !ok {
                        need.lock().unwrap().push(Job { url: format!("{base}{}", crate::api::urlencode(&rel)), rel, dest, size, sha1 });
                    }
                    checked.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }
        let all = futures_util::future::join_all(handles);
        tokio::pin!(all);
        loop {
            tokio::select! {
                _ = &mut all => break,
                _ = tokio::time::sleep(Duration::from_millis(120)) => {
                    let c = checked.load(Ordering::Relaxed);
                    emit(Progress { phase: "check", done: c, total: total_check, files_done: c, files_total: total_check, file: String::new(), speed: 0.0 });
                }
            }
        }
        if self.cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }

        // 2. лишние файлы под update/verify
        if rules.delete_extra {
            let mut local = Vec::new();
            walk(root, root, &mut local);
            for rel in local {
                if !m.files.contains_key(&rel) && rules.should_update(&rel) {
                    let _ = std::fs::remove_file(root.join(&rel));
                }
            }
            remove_empty_dirs(root, root, rules);
        }

        // 3. скачивание
        let jobs: Vec<Job> = std::mem::take(&mut *need.lock().unwrap());
        let total: u64 = jobs.iter().map(|j| j.size).sum();
        let files_total = jobs.len() as u64;
        let done = Arc::new(AtomicU64::new(0));
        let files_done = Arc::new(AtomicU64::new(0));
        let current = Arc::new(Mutex::new(String::new()));
        let started = Instant::now();
        let jobs_stream = futures_util::stream::iter(jobs.into_iter().map(|j| {
            let http = self.http.clone();
            let done = done.clone();
            let files_done = files_done.clone();
            let current = current.clone();
            let cancel = self.cancel.clone();
            let index = self.index.clone();
            async move {
                *current.lock().unwrap() = j.rel.clone();
                let mut last_err = String::new();
                for attempt in 0..3 {
                    if cancel.load(Ordering::Relaxed) {
                        return Err("cancelled".to_string());
                    }
                    let before = done.load(Ordering::Relaxed);
                    match download(&http, &j, &done, &cancel).await {
                        Ok(()) => {
                            if let Some((s, mt)) = stat(&j.dest) {
                                index.lock().unwrap().files.insert(j.dest.to_string_lossy().to_string(), (s, mt, j.sha1.clone()));
                            }
                            files_done.fetch_add(1, Ordering::Relaxed);
                            return Ok(());
                        }
                        Err(e) => {
                            // откатить учтённые байты неудачной попытки
                            let now = done.load(Ordering::Relaxed);
                            done.fetch_sub(now.saturating_sub(before), Ordering::Relaxed);
                            last_err = e;
                            tokio::time::sleep(Duration::from_millis(500 * (attempt + 1))).await;
                        }
                    }
                }
                Err(format!("{}: {}", j.rel, last_err))
            }
        }))
        .buffer_unordered(6);
        tokio::pin!(jobs_stream);
        let mut result: Result<(), String> = Ok(());
        let mut last_emit = Instant::now() - Duration::from_secs(1);
        loop {
            tokio::select! {
                r = jobs_stream.next() => match r {
                    None => break,
                    Some(Err(e)) => { if result.is_ok() { result = Err(e); } self.cancel.store(true, Ordering::Relaxed); }
                    Some(Ok(())) => {}
                },
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
            }
            if last_emit.elapsed() >= Duration::from_millis(100) {
                last_emit = Instant::now();
                let d = done.load(Ordering::Relaxed);
                let secs = started.elapsed().as_secs_f64().max(0.001);
                emit(Progress {
                    phase: "download", done: d, total, files_done: files_done.load(Ordering::Relaxed), files_total,
                    file: current.lock().unwrap().clone(), speed: d as f64 / secs,
                });
            }
        }
        result?;
        emit(Progress { phase: "done", done: total, total, files_done: files_total, files_total, file: String::new(), speed: 0.0 });
        Ok(())
    }
}

async fn download(http: &reqwest::Client, j: &Job, done: &AtomicU64, cancel: &AtomicBool) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    if let Some(p) = j.dest.parent() {
        tokio::fs::create_dir_all(p).await.map_err(|e| e.to_string())?;
    }
    let part = PathBuf::from(format!("{}.cxl-part", j.dest.to_string_lossy()));
    let r = http.get(&j.url).send().await.map_err(|e| e.without_url().to_string())?;
    if !r.status().is_success() {
        return Err(format!("HTTP {}", r.status().as_u16()));
    }
    let mut f = tokio::fs::File::create(&part).await.map_err(|e| e.to_string())?;
    let mut h = Sha1::new();
    let mut got: u64 = 0;
    let mut stream = r.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            drop(f);
            let _ = tokio::fs::remove_file(&part).await;
            return Err("cancelled".into());
        }
        let c = chunk.map_err(|e| e.without_url().to_string())?;
        h.update(&c);
        f.write_all(&c).await.map_err(|e| e.to_string())?;
        got += c.len() as u64;
        done.fetch_add(c.len() as u64, Ordering::Relaxed);
    }
    f.flush().await.map_err(|e| e.to_string())?;
    drop(f);
    let hash = hex::encode(h.finalize());
    if got != j.size || hash != j.sha1 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(format!("контрольная сумма не совпала ({got}/{} байт)", j.size));
    }
    // замена: rename поверх существующего (на Windows MoveFileEx REPLACE_EXISTING)
    if tokio::fs::rename(&part, &j.dest).await.is_err() {
        let _ = tokio::fs::remove_file(&j.dest).await;
        tokio::fs::rename(&part, &j.dest).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            walk(root, &p, out);
        } else if ft.is_file() {
            if let Ok(rel) = p.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if !rel.ends_with(".cxl-part") {
                    out.push(rel);
                }
            }
        }
    }
}

fn remove_empty_dirs(root: &Path, dir: &Path, rules: &Rules) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else { return false };
    let mut empty = true;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if !remove_empty_dirs(root, &p, rules) {
                empty = false;
            }
        } else {
            empty = false;
        }
    }
    if empty && dir != root {
        let rel = dir.strip_prefix(root).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        if rules.should_update(&rel) {
            return std::fs::remove_dir(dir).is_ok();
        }
        return false;
    }
    empty && dir != root
}

pub fn rules_from_profile(p: &serde_json::Value) -> Rules {
    let list = |k: &str| -> Vec<String> {
        p.get(k).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
    };
    Rules { update: list("update"), verify: list("updateVerify"), exclusions: list("updateExclusions"), delete_extra: true, update_all: false }
}

#[allow(dead_code)]
pub fn manifest_paths(m: &Manifest) -> BTreeMap<String, u64> {
    m.files.iter().map(|(k, (s, _))| (k.clone(), *s)).collect()
}
