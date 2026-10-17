// Server List Ping (как PingService/ServerPinger старого лаунчера): онлайн/максимум для карточек серверов.
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[derive(Serialize)]
pub struct PingInfo {
    pub online: i64,
    pub max: i64,
}

fn varint(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            break;
        }
    }
}

async fn read_varint(s: &mut TcpStream) -> std::io::Result<u32> {
    let mut v: u32 = 0;
    for i in 0..5 {
        let b = s.read_u8().await?;
        v |= ((b & 0x7f) as u32) << (7 * i);
        if b & 0x80 == 0 {
            return Ok(v);
        }
    }
    Err(std::io::Error::other("varint"))
}

pub async fn ping(host: &str, port: u16) -> Result<PingInfo, String> {
    timeout(Duration::from_secs(5), ping_inner(host, port))
        .await
        .map_err(|_| "timeout".to_string())?
        .map_err(|e| e.to_string())
}

async fn ping_inner(host: &str, port: u16) -> std::io::Result<PingInfo> {
    // через ретранслятор РФ, если прямой путь режут (route.rs); в рукопожатии остаётся настоящее имя сервера
    let mut s = TcpStream::connect((crate::route::tcp_host(host), port)).await?;
    let mut hs = Vec::new();
    varint(0, &mut hs); // packet id handshake
    varint(767, &mut hs); // протокол 1.21.1
    varint(host.len() as u32, &mut hs);
    hs.extend_from_slice(host.as_bytes());
    hs.extend_from_slice(&port.to_be_bytes());
    varint(1, &mut hs); // next state: status
    let mut pkt = Vec::new();
    varint(hs.len() as u32, &mut pkt);
    pkt.extend_from_slice(&hs);
    pkt.extend_from_slice(&[1, 0]); // status request
    s.write_all(&pkt).await?;
    let _len = read_varint(&mut s).await?;
    let _id = read_varint(&mut s).await?;
    let n = read_varint(&mut s).await? as usize;
    if n > 1 << 20 {
        return Err(std::io::Error::other("too big"));
    }
    let mut buf = vec![0u8; n];
    s.read_exact(&mut buf).await?;
    let v: serde_json::Value = serde_json::from_slice(&buf)?;
    let p = &v["players"];
    Ok(PingInfo { online: p["online"].as_i64().unwrap_or(0), max: p["max"].as_i64().unwrap_or(0) })
}
