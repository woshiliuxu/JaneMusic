//! Jane Music 2.0 —— 随机听歌。
//!
//! 后端只干一件事：代替前端去抓 Meting 兼容接口的歌单 / 歌词，
//! 避开 WebView 的 CORS 限制。播放、界面全在前端。

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// 仓库地址（托盘菜单的 GitHub 项），与前端 constants.js 的 REPO_URL 保持一致
const REPO_URL: &str = "https://github.com/woshiliuxu/JaneMusic";

/// DWM：窗口圆角偏好 / 边框颜色（Windows 11）。迷你胶囊的圆角是 CSS 画的，
/// 系统再裁一次窗口圆角的话，两者半径不一致，四角会露出窗边。
#[cfg(windows)]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(hwnd: isize, attr: u32, pv: *const core::ffi::c_void, cb: u32) -> i32;
}

/// 窗口外观三档：rounded=Some(true) 大/小窗（DWM 强制圆角、无隐形边框）；
/// Some(false) 胶囊（方角）；None 恢复系统默认。
/// shadow 必须关：Tauri 无边框+阴影会保留 WS_THICKFRAME 隐形边框（outer−client=16×9），
/// 那块非客户区被窗口类背景刷刷成纯白，就是窗口左/下边缘的白框。
fn set_flat_frame(win: &tauri::WebviewWindow, rounded: Option<bool>) {
    #[cfg(windows)]
    unsafe {
        if let Ok(h) = win.hwnd() {
            let corner: u32 = match rounded {
                Some(true) => 2,          // DWMWCP_ROUND：强制圆角
                Some(false) => 1,         // DWMWCP_DONOTROUND：胶囊方角
                None => 0,                // DWMWCP_DEFAULT
            };
            let _ = DwmSetWindowAttribute(
                h.0 as isize,
                33, // DWMWA_WINDOW_CORNER_PREFERENCE
                &corner as *const u32 as *const core::ffi::c_void,
                4,
            );
            let border: u32 = if rounded.is_none() { 0xFFFF_FFFF } else { 0xFFFF_FFFE }; // DEFAULT / NONE
            let _ = DwmSetWindowAttribute(
                h.0 as isize,
                34, // DWMWA_BORDER_COLOR
                &border as *const u32 as *const core::ffi::c_void,
                4,
            );
        }
    }
    #[cfg(not(windows))]
    let _ = (win, rounded);
}

const CLIENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);

static STREAM_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
static PART_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// 正在下载的 URL（同一首个连接在拉，第二个连接不要重复拉 12MB）
static INFLIGHT: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

#[derive(Clone, Serialize)]
struct Song {
    title: String,
    author: String,
    url: String,
    /// 备用音源（主源给不出音频时再试一次）
    alt: String,
    pic: String,
    lrc: String,
}

fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(CLIENT)
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| e.to_string())
}

/* ---------------- 本地流式代理 ----------------
   WebView2 的自定义协议（jmstream）喂 <audio> 有坑：PLAYING 后立刻 err1/err4。
   改起一个真 127.0.0.1 HTTP 服务器，两种路径：
   - 已缓存 → 磁盘 + Range，可拖动进度，秒开；
   - 未缓存 → 边下边播（带 Content-Length，播放器不用等整个文件），同时落盘，
     第二次播放走缓存。
   封面图也走这里，必须按魔数判断类型：以前一律当音频校验，JPEG 被封面的
   "not audio" 直接删了，封面永远是占位图。 */

fn hash_url(url: &str) -> u64 {
    url.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    })
}

/// 按文件头判断内容类型：返回 (Content-Type, 是否音频)
fn sniff(head: &[u8]) -> (&'static str, bool) {
    if head.starts_with(b"ID3") || (head.len() >= 2 && head[0] == 0xFF && (head[1] & 0xE0) == 0xE0) {
        return ("audio/mpeg", true);
    }
    if head.starts_with(b"fLaC") {
        return ("audio/flac", true);
    }
    if head.starts_with(b"OggS") {
        return ("audio/ogg", true);
    }
    if head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"WAVE" {
        return ("audio/wav", true);
    }
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ("image/jpeg", false);
    }
    if head.starts_with(&[0x89, b'P', b'N', b'G']) {
        return ("image/png", false);
    }
    if head.starts_with(b"GIF8") {
        return ("image/gif", false);
    }
    if head.len() >= 12 && head.starts_with(b"RIFF") && &head[8..12] == b"WEBP" {
        return ("image/webp", false);
    }
    ("application/octet-stream", false)
}

fn cache_file(dir: &std::path::Path, url: &str) -> std::path::PathBuf {
    dir.join(format!("{:x}.cache", hash_url(url)))
}

fn inflight_has(url: &str) -> bool {
    INFLIGHT
        .lock()
        .map(|v| v.iter().any(|x| x == url))
        .unwrap_or(false)
}

fn inflight_set(url: &str, on: bool) {
    if let Ok(mut v) = INFLIGHT.lock() {
        if on {
            if !v.iter().any(|x| x == url) {
                v.push(url.to_string());
            }
        } else {
            v.retain(|x| x != url);
        }
    }
}

fn respond_text(mut w: std::net::TcpStream, code: u16, msg: &str, body: &str) {
    use std::io::Write;
    let _ = write!(
        w,
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        code,
        msg,
        body.len(),
        body
    );
    let _ = w.flush();
}

/// 解析 Range 头，返回闭区间 [start, end]；None 表示不合法（回 416）
fn parse_range(spec: &str, total: u64) -> Option<(u64, u64)> {
    let s = spec.trim().strip_prefix("bytes=")?.trim();
    let (a, b) = s.split_once('-')?;
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() {
        let n: u64 = b.parse().ok()?;
        if n == 0 {
            return None;
        }
        let start = total.saturating_sub(n);
        return Some((start, total - 1));
    }
    let start: u64 = a.parse().ok()?;
    if start >= total {
        return None;
    }
    let end = if b.is_empty() {
        total - 1
    } else {
        b.parse::<u64>().ok()?.min(total - 1)
    };
    if end < start {
        return None;
    }
    Some((start, end))
}

/// 从磁盘按 Range 服务（缓存命中路径）
fn serve_cached(mut writer: std::net::TcpStream, path: &std::path::Path, range: Option<&str>) {
    use std::io::{Read, Seek, SeekFrom, Write};

    let total = match std::fs::metadata(path) {
        Ok(m) => m.len(),
        Err(_) => {
            respond_text(writer, 404, "Not Found", "nocache");
            return;
        }
    };
    if total == 0 {
        let _ = std::fs::remove_file(path);
        respond_text(writer, 404, "Not Found", "empty");
        return;
    }
    let mut ct = "application/octet-stream";
    if let Ok(mut f) = std::fs::File::open(path) {
        let mut hb = [0u8; 16];
        let n = f.read(&mut hb).unwrap_or(0);
        ct = sniff(&hb[..n]).0;
    }

    let (start, end, status, reason, extra) = match range.and_then(|r| parse_range(r, total)) {
        Some((s, e)) => (
            s,
            e,
            206u16,
            "Partial Content",
            format!("Content-Range: bytes {}-{}/{}\r\n", s, e, total),
        ),
        None => (0, total - 1, 200u16, "OK", String::new()),
    };
    if range.is_some() && status == 200 {
        // Range 不合法（比如超出长度）：按规范回 416
        let _ = write!(
            writer,
            "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            total
        );
        return;
    }
    let len = end - start + 1;
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nAccept-Ranges: bytes\r\nContent-Length: {}\r\n{}Cache-Control: public, max-age=604800, immutable\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status, reason, ct, len, extra
    );
    if writer.write_all(head.as_bytes()).is_err() {
        return;
    }
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    if f.seek(SeekFrom::Start(start)).is_err() {
        return;
    }
    let mut remaining = len;
    let mut buf = [0u8; 65536];
    while remaining > 0 {
        let want = remaining.min(buf.len() as u64) as usize;
        match f.read(&mut buf[..want]) {
            Ok(0) => break,
            Ok(n) => {
                if writer.write_all(&buf[..n]).is_err() {
                    break;
                }
                remaining -= n as u64;
            }
            Err(_) => break,
        }
    }
    let _ = writer.flush();
}

fn start_stream_server() {
    clean_cache();
    let listener = match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(_) => return,
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    let _ = STREAM_PORT.set(port);
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            match conn {
                Ok(s) => {
                    std::thread::spawn(move || handle_stream_conn(s));
                }
                Err(_) => continue,
            }
        }
    });
}

/// 边下边播：先把头部攒够判断类型，再把上游字节一边回给播放器一边写缓存。
/// Ok(true)=已服务；Ok(false)=内容不是音频（假 200 / 占位图 / 空包），没写过响应头；
/// Err=网络层失败（可能已经写了头，不能再回状态码）。
fn stream_through(
    writer: &mut std::net::TcpStream,
    cache_dir: &std::path::Path,
    url: &str,
    need_audio: bool,
) -> Result<bool, String> {
    use std::io::{Read, Write};

    let client = reqwest::blocking::Client::builder()
        .user_agent(CLIENT)
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(None) // 长下载，靠读超时兜不住
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(url).send().map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    eprintln!("[stream] {} -> {}", &url[..url.len().min(90)], status);
    if !(200..300).contains(&status) {
        return Ok(false);
    }
    let total = resp.content_length();
    let mut body = resp;

    let mut buf = vec![0u8; 65536];
    let n = body.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        eprintln!("[stream] empty body, treat as dead");
        return Ok(false);
    }
    let (ct, is_audio) = sniff(&buf[..n]);
    if need_audio && !is_audio {
        eprintln!(
            "[stream] not audio ({:02x?}), treat as dead",
            &buf[..n.min(4)]
        );
        return Ok(false);
    }

    // 响应头：有 Content-Length 就带上，播放器才能算时长/进度。
    // Accept-Ranges 必须声明，否则 Chromium 认为不可 seek，拖进度条会从头重播。
    let head = match total {
        Some(t) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nCache-Control: public, max-age=604800, immutable\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            ct, t
        ),
        None => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nAccept-Ranges: bytes\r\nCache-Control: public, max-age=604800, immutable\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            ct
        ),
    };
    let mut alive = writer.write_all(head.as_bytes()).is_ok();

    // 落盘：唯一临时名，避免两个连接写同一个文件互相破坏
    let seq = PART_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let part = cache_dir.join(format!("{:x}.{}.part", hash_url(url), seq));
    let mut file = std::fs::File::create(&part).ok();
    let mut cache_ok = file.is_some();

    let mut len = n;
    loop {
        if cache_ok {
            let bad = match file.as_mut() {
                Some(f) => f.write_all(&buf[..len]).is_err(),
                None => true,
            };
            if bad {
                cache_ok = false;
                file = None;
                let _ = std::fs::remove_file(&part);
            }
        }
        if alive && writer.write_all(&buf[..len]).is_err() {
            // 播放器走了也别停：把文件下完，下次秒开
            alive = false;
        }
        if !alive && !cache_ok {
            break;
        }
        len = match body.read(&mut buf) {
            Ok(0) => break,
            Ok(k) => k,
            Err(e) => {
                eprintln!("[stream] body err: {}", e);
                drop(file);
                let _ = std::fs::remove_file(&part);
                return Err(e.to_string());
            }
        };
    }
    drop(file);
    if alive {
        let _ = writer.flush();
    }
    if cache_ok {
        let _ = std::fs::rename(&part, cache_file(cache_dir, url));
    }
    Ok(true)
}

/// 缓存还没落地时播放器 seek 会带 Range 来请求。把这段 Range 直接转发给上游
/// （CDN 支持 206），否则只能回 200 全量，一拖进度条就从头开始。
/// 返回 true = 已写出完整响应；false = 上游不支持，交给边下边播走 200。
fn proxy_range(writer: &mut std::net::TcpStream, url: &str, range: &str) -> bool {
    use std::io::{Read, Write};

    let client = match reqwest::blocking::Client::builder()
        .user_agent(CLIENT)
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(None)
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    let resp = match client.get(url).header("range", range).send() {
        Ok(r) => r,
        Err(_) => return false,
    };
    if resp.status().as_u16() != 206 {
        return false;
    }
    let cr = resp
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if cr.is_empty() {
        return false;
    }
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();
    let len = resp.content_length();
    let head = match len {
        Some(n) => format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: {}\r\nAccept-Ranges: bytes\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            ct, cr, n
        ),
        None => format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            ct, cr
        ),
    };
    if writer.write_all(head.as_bytes()).is_err() {
        return true; // 播放器已经走了
    }
    let mut body = resp;
    let mut buf = vec![0u8; 65536];
    loop {
        match body.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if writer.write_all(&buf[..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let _ = writer.flush();
    eprintln!("[stream] range pass {cr} {}", &url[..url.len().min(60)]);
    true
}

fn handle_stream_conn(stream: std::net::TcpStream) {
    use std::io::{BufRead, BufReader};

    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream);

    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() || req_line.is_empty() {
        return;
    }
    let mut range: Option<String> = None;
    loop {
        let mut l = String::new();
        match reader.read_line(&mut l) {
            Ok(0) => break,
            Ok(_) => {
                let lower = l.to_ascii_lowercase();
                if lower.starts_with("range:") {
                    range = l.splitn(2, ':').nth(1).map(|v| v.trim().to_string());
                }
                if l == "\r\n" || l == "\n" {
                    break;
                }
            }
            Err(_) => return,
        }
    }

    let path = req_line.split_whitespace().nth(1).unwrap_or("");
    let query = path.splitn(2, '?').nth(1).unwrap_or("");
    let mut url = String::new();
    let mut alt = String::new();
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("u=") {
            url = percent_decode(v);
        } else if let Some(v) = pair.strip_prefix("alt=") {
            alt = percent_decode(v);
        }
    }
    if !url.starts_with("http") {
        respond_text(writer, 400, "Bad Request", "bad url");
        return;
    }
    // 音频端点（type=url）才做"必须是音频"的校验，封面图放行
    let need_audio = url.contains("type=url") || alt.contains("type=url");

    let cache_dir = std::env::temp_dir().join("jane_music_cache");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cached = cache_file(&cache_dir, &url);

    if cached.exists() {
        serve_cached(writer, &cached, range.as_deref());
        return;
    }

    // 还没缓存（或正在下）时遇到 seek：先把这段转发给上游，保证能真跳到那句
    if let Some(r) = range.as_deref() {
        if proxy_range(&mut writer, &url, r) {
            return;
        }
    }

    // 另一个连接正在拉同一首：等它落地（最多 20 秒）再读磁盘，别重复拉一遍
    if inflight_has(&url) {
        for _ in 0..200 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if cached.exists() {
                serve_cached(writer, &cached, range.as_deref());
                return;
            }
            if !inflight_has(&url) {
                break;
            }
        }
    }

    inflight_set(&url, true);
    let mut result = stream_through(&mut writer, &cache_dir, &url, need_audio);
    // 主源给不出音频（下架 / 假 200 占位图 / 空包）→ 换备用源再试一次
    if matches!(result, Ok(false)) && alt.starts_with("http") && alt != url {
        eprintln!("[stream] fallback -> {}", &alt[..alt.len().min(90)]);
        result = stream_through(&mut writer, &cache_dir, &alt, need_audio);
        if matches!(result, Ok(true)) {
            // 备用源成了，把它也存成主 URL 的缓存，下次直接命中
            let altc = cache_file(&cache_dir, &alt);
            if altc.exists() {
                let _ = std::fs::rename(&altc, &cached);
            }
        }
    }
    inflight_set(&url, false);

    match result {
        Ok(true) => {}
        Ok(false) => respond_text(writer, 404, "Not Found", "gone"),
        Err(e) => {
            eprintln!("[stream] err: {}", e);
            respond_text(writer, 502, "Bad Gateway", "upstream");
        }
    }
}

#[tauri::command]
fn get_stream_port() -> u16 {
    *STREAM_PORT.get().unwrap_or(&0)
}

/* 用系统默认浏览器打开链接（GitHub 仓库 / 新版本下载页） */
#[tauri::command]
fn open_url(url: String) {
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", &url])
        .spawn();
}

/* ---------------- 状态持久化 ----------------
   WebView2 的 localStorage 落盘延迟极长，强杀进程直接丢，
   「恢复上次在听的歌」不能靠它 —— 写 %APPDATA%\JaneMusic\state.json。 */

fn state_path() -> Option<std::path::PathBuf> {
    let base = std::env::var("APPDATA").ok()?;
    let dir = std::path::Path::new(&base).join("JaneMusic");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir.join("state.json"))
}

#[tauri::command]
fn load_state() -> String {
    match state_path().and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(s) => s,
        None => String::new(),
    }
}

#[tauri::command]
fn save_state(json: String) {
    if let Some(p) = state_path() {
        let tmp = p.with_extension("json.tmp");
        if std::fs::write(&tmp, json.as_bytes()).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }
}

/// 后台预取几首歌进磁盘缓存（串行、限量，别像压测）。
/// 前端启动/换榜单时调，用户之后随机抽到这些歌就是秒开。
#[tauri::command]
fn prefetch(urls: Vec<String>) {
    std::thread::spawn(move || {
        let dir = std::env::temp_dir().join("jane_music_cache");
        let _ = std::fs::create_dir_all(&dir);
        for url in urls.iter().take(3) {
            if !url.starts_with("http") {
                continue;
            }
            if cache_file(&dir, url).exists() || inflight_has(url) {
                continue;
            }
            inflight_set(url, true);
            let r = download_to_cache(url, &dir);
            inflight_set(url, false);
            eprintln!("[prefetch] {} ok={}", &url[..url.len().min(80)], r.is_ok());
            // 首尾留点间隔，别一口气连下
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    });
}

/// blocking 下载整首歌到缓存（校验音频魔数，占位图/空包不留）。
fn download_to_cache(url: &str, cache_dir: &std::path::Path) -> Result<(), String> {
    use std::io::{Read, Write};
    let client = reqwest::blocking::Client::builder()
        .user_agent(CLIENT)
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(None)
        .build()
        .map_err(|e| e.to_string())?;
    let mut resp = client.get(url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("status {}", resp.status()));
    }
    let mut buf = vec![0u8; 65536];
    let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("empty".into());
    }
    let (_, is_audio) = sniff(&buf[..n]);
    if !is_audio {
        return Err("not audio".into());
    }
    let seq = PART_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let part = cache_dir.join(format!("{:x}.{}.part", hash_url(url), seq));
    let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    loop {
        match resp.read(&mut buf) {
            Ok(0) => break,
            Ok(k) => {
                if file.write_all(&buf[..k]).is_err() {
                    drop(file);
                    let _ = std::fs::remove_file(&part);
                    return Err("disk write".into());
                }
            }
            Err(e) => {
                drop(file);
                let _ = std::fs::remove_file(&part);
                return Err(e.to_string());
            }
        }
    }
    drop(file);
    std::fs::rename(&part, cache_file(cache_dir, url)).map_err(|e| e.to_string())
}

/// 缓存清理：.part 直接删；7 天内的缓存保留（下次启动秒开）；
/// 总量超过 150MB 时从最旧开始删，删到 100MB 以下。
fn clean_cache() {
    let cache_dir = std::env::temp_dir().join("jane_music_cache");
    let now = std::time::SystemTime::now();
    let mut files: Vec<(std::path::PathBuf, u64, std::time::SystemTime)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for e in entries.flatten() {
            let p = e.path();
            let is_part = p.extension().map(|x| x == "part").unwrap_or(false);
            if is_part {
                let _ = std::fs::remove_file(&p);
                continue;
            }
            if let Ok(m) = e.metadata() {
                let mt = m.modified().unwrap_or(now);
                let age = now.duration_since(mt).map(|d| d.as_secs()).unwrap_or(0);
                if age > 7 * 86400 {
                    let _ = std::fs::remove_file(&p);
                } else {
                    files.push((p, m.len(), mt));
                }
            }
        }
    }
    const CAP: u64 = 150 * 1024 * 1024;
    const FLOOR: u64 = 100 * 1024 * 1024;
    let total: u64 = files.iter().map(|f| f.1).sum();
    if total > CAP {
        files.sort_by_key(|f| f.2);
        let mut left = total;
        for (p, sz, _) in files {
            if left <= FLOOR {
                break;
            }
            if std::fs::remove_file(&p).is_ok() {
                left -= sz;
            }
        }
    }
}

/* ---------------- 接口源测速 ----------------
   两个 Meting 兼容源（qijieya / i-meto）。应用一打开就并发各探一次，
   谁先回完整响应就用谁；之后每 10 分钟后台复测，接口临场变卡自动切走。
   歌单拉取顺序、封面/歌词/音频地址重写，全部跟首选源走。 */

const SRC_QIJIEYA: usize = 0;
const SRC_IMETO: usize = 1;
/// 首选源结论的有效期：过期后台复测（不阻塞当前拉歌单，先用旧结论）
const SRC_TTL: u64 = 10 * 60 * 1000;
/// 单源探测上限：挂着不动的源别拖太久（i-meto 挂掉那次就是一直不回话）
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2000);
/// 探针：拉一首歌的歌词，响应小不吃带宽；id 随便挑一首冷门但确实有词的
const PROBE_ID: &str = "186016";

static SRC_PREF: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(SRC_QIJIEYA);
static SRC_AT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PROBE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn src_pref() -> usize {
    SRC_PREF.load(std::sync::atomic::Ordering::Acquire)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 按源编号拼 Meting 接口地址（两个源的路径拼法不一样）。
/// ty：playlist / url / pic / lrc。
fn meting_url(src: usize, ty: &str, id: &str) -> String {
    if src == SRC_IMETO {
        format!("https://api.i-meto.com/meting/api?server=netease&type={ty}&id={id}")
    } else {
        format!("https://api.qijieya.cn/meting/?server=netease&type={ty}&id={id}")
    }
}

/// 探一个源：拉探针歌词，等正文到齐才计时（只到响应头会让"秒回空包"的源占便宜）。
/// 失败 / 超时 / 非成功状态一律返回 None。
async fn probe_one(src: usize) -> Option<u128> {
    let client = reqwest::Client::builder()
        .user_agent(CLIENT)
        .timeout(PROBE_TIMEOUT)
        .build()
        .ok()?;
    let t0 = std::time::Instant::now();
    let resp = client.get(meting_url(src, "lrc", PROBE_ID)).send().await.ok()?;
    let st = resp.status();
    resp.text().await.ok()?;
    if !st.is_success() {
        return None;
    }
    Some(t0.elapsed().as_millis())
}

/// 并发探两个源，快的当选；一边挂了用另一边，两边都挂维持现状。
async fn measure() {
    let (a, b) = tokio::join!(probe_one(SRC_QIJIEYA), probe_one(SRC_IMETO));
    let pref = match (a, b) {
        (Some(x), Some(y)) => {
            if y < x {
                SRC_IMETO
            } else {
                SRC_QIJIEYA // 打平也算 qijieya，保持默认稳定
            }
        }
        (Some(_), None) => SRC_QIJIEYA,
        (None, Some(_)) => SRC_IMETO,
        (None, None) => src_pref(), // 都探不动：沿用旧结论（默认 qijieya）
    };
    SRC_PREF.store(pref, std::sync::atomic::Ordering::Release);
    SRC_AT.store(now_ms(), std::sync::atomic::Ordering::Release);
    eprintln!(
        "[src-probe] qijieya={:?} i-meto={:?} -> pref={pref}",
        a, b
    );
}

/// 确保手上有份没过期的测速结论。从没量过就同步等一次（开局第一拉前必须知道
/// 谁快）；量过但过期就丢后台复测，当前请求用旧结论不阻塞。
async fn ensure_probe() -> usize {
    let at = SRC_AT.load(std::sync::atomic::Ordering::Acquire);
    if at != 0 {
        if now_ms().saturating_sub(at) < SRC_TTL {
            return src_pref();
        }
        // 结论过期：拿到锁的那个去后台复测，其他请求照旧走
        if let Ok(guard) = PROBE_LOCK.try_lock() {
            measure().await;
            drop(guard);
        }
        return src_pref();
    }
    // 首测：加锁防并发重复量；等锁回来先看别人是不是已经量完了
    let guard = PROBE_LOCK.lock().await;
    if SRC_AT.load(std::sync::atomic::Ordering::Acquire) != 0 {
        return src_pref();
    }
    measure().await;
    drop(guard);
    src_pref()
}

/// 前端启动时问一句当前首选源（0=qijieya 1=i-meto），
/// 缓存和 state.json 里存的旧源地址好跟着重写。只读静态值，不触发探测。
#[tauri::command]
fn preferred_source() -> usize {
    src_pref()
}

/// 拉一个歌单（网易歌单 id），按启动测速的结论排先后，依次尝试两个 Meting 兼容源。
/// 拉到后并发预筛一遍死链（这个源的 404 率过半，不预筛前端要连环跳很多次）。
#[tauri::command]
async fn fetch_playlist(id: String) -> Result<Vec<Song>, String> {
    let client = http()?;
    // 开局第一拉会在这里等首测结果（setup 里已经提前开探，通常早量完了）
    let first = ensure_probe().await;
    let second = if first == SRC_QIJIEYA { SRC_IMETO } else { SRC_QIJIEYA };
    let sources = [
        meting_url(first, "playlist", &id),
        meting_url(second, "playlist", &id),
    ];
    let mut last_err = String::from("no source");
    for src in &sources {
        let resp = client.get(src).send().await;
        let body = match resp {
            Ok(r) => match r.text().await {
                Ok(t) => t,
                Err(e) => {
                    last_err = e.to_string();
                    continue;
                }
            },
            Err(e) => {
                last_err = e.to_string();
                continue;
            }
        };
        let parsed = parse_meting(&body);
        if !parsed.is_empty() {
            return Ok(prefilter_dead(parsed).await);
        }
        last_err = "empty or invalid response".into();
    }
    Err(last_err)
}

/// 缓存/历史会话里存的 pic/lrc 地址可能来自任一源（源会随测速结果切换），
/// 统一重写到当前首选源；已经是首选的或非 Meting 地址原样返回。
fn fix_meting_url(u: &str) -> String {
    let pref = src_pref();
    let is_imeto = u.contains("api.i-meto.com");
    let is_qijieya = u.contains("api.qijieya.cn");
    if (!is_imeto && !is_qijieya) || is_imeto == (pref == SRC_IMETO) {
        return u.to_string();
    }
    let id = u.split(['?', '&']).find_map(|p| {
        p.strip_prefix("id=").filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
    });
    let ty = u.split(['?', '&'])
        .find_map(|p| p.strip_prefix("type=").filter(|v| matches!(*v, "url" | "pic" | "lrc")));
    match (id, ty) {
        (Some(id), Some(ty)) => meting_url(pref, ty, &id),
        _ => u.to_string(),
    }
}

/// 把歌单里的音频地址重写到当前首选源。qijieya 直接 302 到网易 CDN 最稳，
/// 首选就是它时原样保留；换源后旧地址跟着切，原地址存进 alt 兜底。
fn rewrite_audio_url(url: &str) -> String {
    let id = url
        .split(['?', '&'])
        .find_map(|p| p.strip_prefix("id=").filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())));
    match id {
        Some(id) => meting_url(src_pref(), "url", &id),
        None => url.to_string(),
    }
}

/// 并发探测每首歌的音频地址（Range 0-1，不下载正文），
/// 确认 404/410/401 的直接剔除；探测失败/其他状态的保留，交给前端运行时再验。
async fn prefilter_dead(songs: Vec<Song>) -> Vec<Song> {
    let songs: Vec<Song> = songs
        .into_iter()
        .map(|mut s| {
            let orig = s.url.clone();
            s.url = rewrite_audio_url(&orig);
            s.alt = if s.url == orig { String::new() } else { orig };
            s
        })
        .collect();
    let checker = match reqwest::Client::builder()
        .user_agent(CLIENT)
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return songs, // 客户端都建不出来就别筛了
    };
    let mut out = Vec::with_capacity(songs.len());
    // 只预筛前 100 首（够随机了），剩下的原样保留交给运行时验证；
    // 每批 5 个、批间隔 80ms —— 200 首一次性并发探测太像压测，容易被曲库盯上。
    let split = songs.len().min(100);
    let (head, tail) = songs.split_at(split);
    for chunk in head.chunks(5) {
        let handles: Vec<_> = chunk
            .iter()
            .map(|s| {
                let c = checker.clone();
                let url = s.url.clone();
                tauri::async_runtime::spawn(async move {
                    match c.get(&url).header("range", "bytes=0-1").send().await {
                        Ok(r) => r.status().as_u16(),
                        Err(_) => 0u16,
                    }
                })
            })
            .collect();
        for (s, h) in chunk.iter().zip(handles) {
            let st = h.await.unwrap_or(0);
            let alive = matches!(st, 0 | 200 | 206 | 416);
            if alive {
                out.push(s.clone());
            }
        }
        tokio_sleep(80).await;
    }
    out.extend(tail.iter().cloned());
    // 全军覆没说明是限流不是真死，原样返回
    if out.is_empty() { songs } else { out }
}

async fn tokio_sleep(ms: u64) {
    tauri::async_runtime::spawn_blocking(move || std::thread::sleep(std::time::Duration::from_millis(ms)))
        .await
        .ok();
}

fn parse_meting(body: &str) -> Vec<Song> {
    let arr: Vec<serde_json::Value> = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    arr.into_iter()
        .filter_map(|v| {
            let title = v.get("title")?.as_str()?.to_string();
            let url = v.get("url")?.as_str()?.to_string();
            if url.is_empty() {
                return None;
            }
            Some(Song {
                title,
                author: v
                    .get("author")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                url,
                alt: String::new(),
                pic: v
                    .get("pic")
                    .and_then(|x| x.as_str())
                    .map(fix_meting_url)
                    .unwrap_or_default(),
                lrc: v
                    .get("lrc")
                    .and_then(|x| x.as_str())
                    .map(fix_meting_url)
                    .unwrap_or_default(),
            })
        })
        .collect()
}

/// 抓一段文本（歌词用）。
#[tauri::command]
async fn fetch_text(url: String) -> Result<String, String> {
    let client = http()?;
    let r = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    r.text().await.map_err(|e| e.to_string())
}

/// 验歌：跟随 302 后看最终状态码 + 内容魔数（有的源对死歌返回 200 + 占位图）。
/// 前端只在确认 404/410/415 时才把歌记进死链。
#[tauri::command]
async fn check_url(url: String) -> Result<u16, String> {
    let client = http()?;
    let resp = client
        .get(&url)
        .header("range", "bytes=0-15")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let st = resp.status().as_u16();
    eprintln!("[check_url] {} -> {}", &url[..url.len().min(90)], st);
    if !(200..300).contains(&st) {
        return Ok(st);
    }
    let bytes = resp.bytes().await.unwrap_or_default();
    let looks_audio = bytes.starts_with(b"ID3")
        || (bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0);
    if looks_audio {
        Ok(st)
    } else {
        Ok(415) // Unsupported Media Type：内容不是音频
    }
}

/// 前端调试日志回灌到 stderr（排查播放问题时不用截图看面板）
#[tauri::command]
fn log_line(msg: String) {
    eprintln!("[ui] {}", msg);
}

/// 无边框窗口的自绘按钮走自定义命令（不受 capabilities 权限限制）。
#[tauri::command]
fn minimize_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.minimize();
    }
}

/// 标题栏任意处按下即拖动。小窗标题栏被窗口按钮挤满，Tauri 自带的
/// data-tauri-drag-region 依赖的空白区几乎为 0，改由前端 mousedown 统一调这里。
#[tauri::command]
fn drag_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.start_dragging();
    }
}

/// 一键更新：把新 exe 下到程序同目录 → 校验 PE 头 → 运行中的旧 exe 改名让位 → 换上新包
/// → 2 秒后由 cmd 拉起新实例（等本进程退出，避免被单实例守卫转给旧进程）→ 本进程退出。
/// 任何一步失败都返回 Err，前端回退成「打开下载页手动下」。
///
/// 必须是 async 命令：Tauri 的同步命令跑在主线程上，十几 MB 的阻塞下载会把整个
/// 界面卡成「未响应」——真机就是么挂的。所以这里只当调度员，重活全丢进
/// spawn_blocking，下载进度还通过 jm-upd-progress 事件推给前端画百分比。
#[tauri::command]
async fn update_app(app: AppHandle, url: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || do_update(&app, url))
        .await
        .map_err(|e| format!("更新线程挂了：{e}"))?
}

fn do_update(app: &AppHandle, url: String) -> Result<(), String> {
    use std::io::{Read, Write};

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("程序路径异常")?.to_path_buf();
    let tmp = dir.join("jane_update.tmp");

    let client = reqwest::blocking::Client::builder()
        .user_agent(CLIENT)
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(None) // 十几 MB 的包，别用固定超时卡死
        .build()
        .map_err(|e| e.to_string())?;

    /* 直连 GitHub 下载偶尔抽风，给两次机会 */
    let mut resp = None;
    for i in 0..2 {
        match client.get(&url).send() {
            Ok(r) if r.status().is_success() => {
                resp = Some(r);
                break;
            }
            Ok(r) => {
                let code = r.status();
                if i == 1 {
                    return Err(format!("下载失败 HTTP {}", code));
                }
            }
            Err(e) => {
                if i == 1 {
                    return Err(format!("连不上下载地址：{}", e));
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let mut resp = resp.ok_or("下载失败")?;

    let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    let total = resp.content_length().unwrap_or(0);
    let mut buf = vec![0u8; 256 * 1024];
    let mut got: u64 = 0;
    let mut last_pct: u32 = 0;
    loop {
        let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        got += n as u64;
        /* 进度推给前端（NEW 徽标变百分比），每变 1% 才发一条，别刷屏 */
        if total > 0 {
            let pct = ((got * 100) / total) as u32;
            if pct != last_pct {
                last_pct = pct;
                let _ = app.emit("jm-upd-progress", pct);
            }
        }
    }
    drop(f);
    let _ = app.emit("jm-upd-progress", 100);

    // 校验确实是 Windows 可执行文件（MZ 头）：代理/网络出错时可能下到 HTML 错误页
    let mut head = [0u8; 2];
    std::fs::File::open(&tmp)
        .and_then(|mut x| x.read_exact(&mut head))
        .map_err(|e| e.to_string())?;
    if &head != b"MZ" {
        let _ = std::fs::remove_file(&tmp);
        return Err("下载到的不是可执行文件".into());
    }

    // Windows 不允许覆盖正在运行的 exe，但允许改名（运行中的进程不受影响）
    let old = dir.join("jane_update.old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| format!("旧包让位失败：{}", e))?;
    if let Err(e) = std::fs::rename(&tmp, &exe) {
        let _ = std::fs::rename(&old, &exe); // 换不进去就滚回去，旧版照常能用
        return Err(format!("替换失败：{}", e));
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        /* 用 raw_arg 原样传命令行：走普通 args() 时 std 会把整串加引号并转义内层引号，
           cmd 解析 `start "" "路径"` 就会坏掉（表现为替换成功但新进程没起来）。
           延时 2 秒是等本进程退出、放开单实例互斥量，否则新实例会被转给正在退出的旧实例。 */
        let line = format!("/C timeout /t 2 /nobreak >nul & start \"\" \"{}\"", exe.display());
        let _ = std::process::Command::new("cmd")
            .raw_arg(line)
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW：别闪一个黑框
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new(&exe).spawn();
    }

    app.exit(0);
    Ok(())
}

/// 点关闭 = 隐藏到托盘（音乐继续放），真正退出走托盘菜单。
#[tauri::command]
fn close_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// 托盘悬停气泡：正在放歌显示「歌名 - 歌手」，没在放就显示前端挑的一句趣味提示。
/// 文案全由前端决定（换歌 / 暂停 / 定时轮换都会推一次），这里只管贴上去。
#[tauri::command]
fn set_tray_tip(app: AppHandle, text: String) {
    if let Some(tray) = app.tray_by_id("jane-tray") {
        let _ = tray.set_tooltip(Some(text));
    }
}

/// 标题栏图钉：切换窗口置顶。返回置顶后的状态。
#[tauri::command]
fn toggle_pin(app: AppHandle) -> bool {
    if let Some(w) = app.get_webview_window("main") {
        let top = w.is_always_on_top().unwrap_or(false);
        let _ = w.set_always_on_top(!top);
        return !top;
    }
    false
}

/// 指定置顶状态（迷你模式进出时用，避免靠 toggle 猜当前值）
#[tauri::command]
fn set_pin(app: AppHandle, on: bool) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_always_on_top(on);
    }
}

#[tauri::command]
fn get_pin(app: AppHandle) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.is_always_on_top().ok())
        .unwrap_or(false)
}

/// 切换窗口形态（逻辑像素，与 DPI 无关）。三个固定档位，都不允许拖边缩放：
/// flat=true  胶囊 340x84：摘阴影、DWM 去圆角去边框，四角完全交给 CSS；
/// flat=false 大窗/小窗：保留系统阴影，resizable 照样关掉（WS_THICKFRAME 会留隐形边）。
#[tauri::command]
fn set_window_size(app: AppHandle, w: f64, h: f64, rounded: bool) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_min_size(None::<tauri::Size>);
        let _ = win.set_resizable(false);
        // 统一摘掉阴影：shadow(true) 会在窗口四周留一圈被刷成白色的非客户区（白框）。
        // 大/小窗用 DWM 强制圆角补回窗口轮廓，胶囊保持方角（圆角由 CSS 画）。
        let _ = win.set_shadow(false);
        set_flat_frame(&win, Some(rounded));
        let _ = win.set_size(tauri::LogicalSize::new(w, h));
    }
}

/// 托盘切模式：唤起窗口并给前端发一条模式请求（布局与状态都在前端管）。
fn request_mode(app: &AppHandle, mode: &str) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        let _ = w.emit("jm-mode", mode);
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    let hex = |c: u8| (c as char).to_digit(16);
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() && hex(b[i + 1]).is_some() && hex(b[i + 2]).is_some() => {
                out.push(((hex(b[i + 1]).unwrap() * 16 + hex(b[i + 2]).unwrap()) as u8));
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    start_stream_server();
    /* 一键更新留下的旧包（13MB）：新进程起来了说明它已经不被占用，能删就删 */
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            let _ = std::fs::remove_file(d.join("jane_update.old"));
        }
    }
    let builder = tauri::Builder::default();
    // 双开守卫：已有实例时唤起它的窗口然后退出（两个实例共用 WebView2 数据目录会互相干扰）。
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        show_main(app);
    }));
    builder
        .setup(|app| {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
            };

            let show = MenuItem::with_id(app, "show", "把窗口叫出来", true, None::<&str>)?;
            let next = MenuItem::with_id(app, "next", "下一曲", true, None::<&str>)?;
            let big = MenuItem::with_id(app, "mode-big", "大窗待着", true, None::<&str>)?;
            let small = MenuItem::with_id(app, "mode-small", "缩成小窗", true, None::<&str>)?;
            let mini = MenuItem::with_id(app, "mode-mini", "缩成小胶囊", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let pin = MenuItem::with_id(app, "pin", "钉在最前面", true, None::<&str>)?;
            /* 版本号只是一条展示，点不了（enabled=false 会显示成灰字） */
            let ver = MenuItem::with_id(
                app,
                "ver",
                concat!("当前版本 v", env!("CARGO_PKG_VERSION")),
                false,
                None::<&str>,
            )?;
            let check = MenuItem::with_id(app, "check-upd", "检查更新", true, None::<&str>)?;
            let github = MenuItem::with_id(app, "github", "GitHub", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出 Jane Music", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &show, &sep, &next, &big, &small, &mini, &sep, &pin, &sep, &ver, &check,
                    &github, &sep, &quit,
                ],
            )?;

            // 三个档位都是固定尺寸（前端 set_window_size 管），不再需要最小尺寸限制
            // 窗口标题带上版本号（任务栏 / Alt+Tab 可见），版本从 Cargo.toml 自动取，
            // 以后改版本号不用再回来改这里
            let full_name = concat!("Jane Music ", env!("CARGO_PKG_VERSION"));
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_min_size(None::<tauri::Size>);
                let _ = w.set_title(full_name);
            }

            TrayIconBuilder::with_id("jane-tray")
                .icon(app.default_window_icon().expect("no default icon").clone())
                .tooltip(full_name)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id.as_ref() {
                    "show" => show_main(app),
                    /* 切歌/查更新都交给前端：播放逻辑和更新流程都在那边 */
                    "next" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.emit("jm-tray", "next");
                        }
                    }
                    "check-upd" => {
                        show_main(app);
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.emit("jm-tray", "check");
                        }
                    }
                    "github" => open_url(REPO_URL.to_string()),
                    "mode-big" => request_mode(app, "big"),
                    "mode-small" => request_mode(app, "small"),
                    "mode-mini" => request_mode(app, "mini"),
                    "pin" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let top = w.is_always_on_top().unwrap_or(false);
                            let _ = w.set_always_on_top(!top);
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            // 开局就把两个接口源量一遍：跟 WebView 加载并行，前端第一拉歌单时
            // 结论多半已经好了，不用再等
            tauri::async_runtime::spawn(async {
                ensure_probe().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_playlist,
            preferred_source,
            fetch_text,
            check_url,
            get_stream_port,
            open_url,
            load_state,
            save_state,
            prefetch,
            log_line,
            minimize_window,
            drag_window,
            update_app,
            close_window,
            set_tray_tip,
            toggle_pin,
            set_pin,
            get_pin,
            set_window_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running Jane Music");
}
