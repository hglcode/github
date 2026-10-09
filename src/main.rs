use futures::TryStreamExt;
use futures::stream::{self, StreamExt};
use regex::Regex;
use reqwest::header;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

// [新增] 带宽探测常量配置
const MAX_PROBE_BYTES: u64 = 256 * 1024; // 最多读取 256KB 用于测速
const PROBE_TIMEOUT: Duration = Duration::from_secs(3); // 单节点测速最长耗时

/// 运行配置。所有字段都能通过同名 `GMIRROR_*` 环境变量覆盖默认值
#[derive(Debug, Clone)]
struct Config {
    site_origin: String,
    fallback_chunk: String,
    probe_path: String,
    concurrency: usize,
    chunk_re: String,
    mirror_re: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            site_origin: "https://github.akams.cn".into(),
            fallback_chunk: "https://github.akams.cn/_next/static/chunks/096f2b9eccbdd6b4.js"
                .into(),
            probe_path: "https://raw.githubusercontent.com/microsoft/vscode/main/package-lock.json"
                .into(),
            concurrency: 16,
            chunk_re: r#"/_next/static/chunks/[A-Za-z0-9_\-]+\.js"#.into(),
            mirror_re: r#"\{\s*label:\s*"[^"]+?",\s*value:\s*"([^"]+?)"\s*\}"#.into(),
        }
    }
}

impl Config {
    fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = env::var("GMIRROR_ORIGIN") {
            cfg.site_origin = v.trim_end_matches('/').to_string();
        }
        if let Ok(v) = env::var("GMIRROR_FALLBACK_CHUNK") {
            cfg.fallback_chunk = v;
        }
        if let Ok(v) = env::var("GMIRROR_PROBE") {
            cfg.probe_path = v;
        }
        if let Ok(v) = env::var("GMIRROR_CONCURRENCY")
            && let Ok(n) = v.parse::<usize>()
            && n > 0
        {
            cfg.concurrency = n;
        }
        if let Ok(v) = env::var("GMIRROR_CHUNK_RE") {
            cfg.chunk_re = v;
        }
        if let Ok(v) = env::var("GMIRROR_MIRROR_RE") {
            cfg.mirror_re = v;
        }
        cfg
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Config::from_env();
    let args: Vec<String> = env::args().collect();
    let uri = args.get(1).cloned();

    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0 Safari/537.36",
        )
        .default_headers({
            let mut h = header::HeaderMap::new();
            h.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
            h
        })
        .timeout(Duration::from_secs(10))
        .build()?;

    let mirrors = fetch_mirrors(&client, &cfg).await?;
    if mirrors.is_empty() {
        eprintln!("未找到任何镜像");
        return Ok(());
    }
    eprintln!("发现 {} 个镜像，开始带宽测速...", mirrors.len());

    let raw_probe = uri.clone().unwrap_or_else(|| cfg.probe_path.clone());
    let probe_path = normalize_probe_path(&raw_probe);

    // [重构] 测速返回结果改为 (Mirror URL, 速度 Bytes/s)
    let results: Vec<Option<(String, f64)>> = stream::iter(mirrors.into_iter().map(|m| {
        let client = client.clone();
        let probe_path = probe_path.clone();
        async move {
            let url = format!("{}/{}", m.trim_end_matches('/'), probe_path);

            // 使用 tokio::time::timeout 包裹，防止低速节点阻塞并发池
            match tokio::time::timeout(PROBE_TIMEOUT, measure_bandwidth(&client, &url)).await {
                Ok(Ok(speed)) => Some((m, speed)),
                Ok(Err(e)) => {
                    eprintln!("  [err  ] {}: {}", m, e);
                    None
                }
                Err(_) => {
                    eprintln!("  [slow ] {}: 测速超时 (>{:?})", m, PROBE_TIMEOUT);
                    None
                }
            }
        }
    }))
    .buffer_unordered(cfg.concurrency)
    .collect()
    .await;

    // [重构] 按带宽降序排列
    let mut ok: Vec<(String, f64)> = results.into_iter().flatten().collect();
    ok.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if !ok.is_empty() {
        eprintln!("--- 测速结果 (按带宽降序) ---");
        for (m, speed) in &ok {
            if *speed >= 1024.0 * 1024.0 {
                eprintln!("  ✓ {:<40} {:.2} MB/s", m, speed / (1024.0 * 1024.0));
            } else {
                eprintln!("  ✓ {:<40} {:.2} KB/s", m, speed / 1024.0);
            }
        }
        eprintln!("----------------------------");
    }

    let mut links: Vec<String> = match &uri {
        Some(_) => ok
            .into_iter()
            .map(|(m, _)| format!("{}/{}", m.trim_end_matches('/'), probe_path))
            .collect(),
        None => ok.into_iter().map(|(m, _)| m).collect(),
    };
    if let Some(u) = &uri {
        links.push(u.clone());
    }

    if links.is_empty() {
        eprintln!("所有镜像均不可达或带宽过低");
        return Ok(());
    }

    println!("{}", links.join(" "));
    Ok(())
}

/// [新增] 实际下载数据并计算带宽 (Bytes/sec)
/// 最多读取 MAX_PROBE_BYTES 字节后立即中断，避免浪费流量
async fn measure_bandwidth(
    client: &reqwest::Client,
    url: &str,
) -> Result<f64, Box<dyn Error + Send + Sync>> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let mut stream = resp.bytes_stream(); // 启用 stream feature 后即可正常调用

    let start = Instant::now();
    let mut total_bytes: u64 = 0;

    // try_next() 自动解包 Result，chunk 类型被明确推断为 Bytes
    while let Some(chunk) = stream.try_next().await? {
        total_bytes += chunk.len() as u64;
        if total_bytes >= MAX_PROBE_BYTES {
            break;
        }
    }

    let elapsed = start.elapsed();
    if elapsed.as_secs_f64() == 0.0 || total_bytes == 0 {
        return Err("零字节或零耗时".into());
    }

    Ok(total_bytes as f64 / elapsed.as_secs_f64())
}

fn normalize_probe_path(raw: &str) -> String {
    if (raw.starts_with("http://") || raw.starts_with("https://"))
        && let Some(idx) = raw.find("://")
    {
        let after_scheme = &raw[idx + 3..];
        if let Some(path_idx) = after_scheme.find('/') {
            return after_scheme[path_idx + 1..].to_string();
        }
    }
    raw.trim_start_matches('/').to_string()
}

async fn fetch_mirrors(
    client: &reqwest::Client,
    cfg: &Config,
) -> Result<Vec<String>, Box<dyn Error>> {
    let chunk_re = Regex::new(&cfg.chunk_re)?;
    let mirror_re = Regex::new(&cfg.mirror_re)?;

    let mut candidates: Vec<String> = Vec::new();

    if let Ok(resp) = client.get(format!("{}/", cfg.site_origin)).send().await
        && let Ok(resp) = resp.error_for_status()
        && let Ok(html) = resp.text().await
    {
        for cap in chunk_re.captures_iter(&html) {
            let path = cap.get(0).unwrap().as_str();
            let url = if path.starts_with("http://") || path.starts_with("https://") {
                path.to_string()
            } else {
                format!("{}{}", cfg.site_origin, path)
            };
            candidates.push(url);
        }
    } else {
        eprintln!("warn: 抓取首页失败，将直接尝试 fallback chunk");
    }

    candidates.sort();
    candidates.dedup();
    candidates.push(cfg.fallback_chunk.clone());

    for url in &candidates {
        let resp = match client.get(url).send().await {
            Ok(r) => match r.error_for_status() {
                Ok(r) => r,
                Err(_) => continue,
            },
            Err(_) => continue,
        };
        let body = match resp.text().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        let mirrors: Vec<String> = mirror_re
            .captures_iter(&body)
            .filter_map(|c| {
                let raw = c.get(1)?.as_str().trim_end_matches('/');
                let url = if raw.starts_with("http://") || raw.starts_with("https://") {
                    raw.to_string()
                } else {
                    format!("https://{}", raw)
                };
                Some(url)
            })
            .collect();

        if !mirrors.is_empty() {
            eprintln!("已从 {} 抠到 {} 个镜像", url, mirrors.len());
            return Ok(mirrors);
        }
    }
    Err("未能在任何 chunk 中找到镜像列表，站点结构可能已变更".into())
}
