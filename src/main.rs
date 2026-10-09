use futures::stream::{self, StreamExt};
use regex::Regex;
use reqwest::header;
use std::env;
use std::error::Error;
use std::time::{Duration, Instant};

/// 运行配置。所有字段都能通过同名 `GMIRROR_*` 环境变量覆盖默认值：
///
/// - `GMIRROR_ORIGIN`         站点源，默认 https://github.akams.cn
/// - `GMIRROR_FALLBACK_CHUNK` 首页发现失败时兜底的 chunk URL
/// - `GMIRROR_PROBE`          不传参数时用来探测镜像可用性的文件
/// - `GMIRROR_CONCURRENCY`     并发测活上限，默认 16
/// - `GMIRROR_CHUNK_RE`       从首页 HTML 里找 chunk 的正则
/// - `GMIRROR_MIRROR_RE`      从 chunk JS 里抠 mirror value 的正则（须含 1 个捕获组）
///
/// 注意：本代码使用了 let-chain 语法，要求 Rust 1.82+ (MSRV: 1.82)
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
            probe_path: "https://raw.githubusercontent.com/torvalds/linux/master/README".into(),
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
            // [修复] 修正非法的 MIME 类型
            h.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
            h
        })
        .timeout(Duration::from_secs(10))
        .build()?;

    // 1. 自动发现 chunk，抠出 mirror 列表
    let mirrors = fetch_mirrors(&client, &cfg).await?;
    if mirrors.is_empty() {
        eprintln!("未找到任何镜像");
        return Ok(());
    }
    eprintln!("发现 {} 个镜像，开始测活...", mirrors.len());

    // 2. 探测目标：传了命令行参数就用它，否则用内置 probe
    let raw_probe = uri.clone().unwrap_or_else(|| cfg.probe_path.clone());
    // [修复] 规范化 probe 路径，防止用户传入完整 URL 导致拼接错误
    let probe_path = normalize_probe_path(&raw_probe);

    // 3. 限并发测活，记录每个成功响应的 TTFB (首字节时间)
    let results: Vec<Option<(String, Duration)>> = stream::iter(mirrors.into_iter().map(|m| {
        let client = client.clone();
        let probe_path = probe_path.clone();
        async move {
            // 确保拼接时中间只有一个 '/'
            let url = format!("{}/{}", m.trim_end_matches('/'), probe_path);
            let start = Instant::now();

            // [优化] 只发送到收到 Header 为止 (TTFB)，不下载整个 body，大幅提升大文件测速效率
            match client.get(&url).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(_) => {
                        // 收到 header 即视为连通，记录耗时后立即丢弃 response 中断下载
                        let elapsed = start.elapsed();
                        Some((m, elapsed))
                    }
                    Err(e) => {
                        eprintln!("  [http ] {}: {}", m, e);
                        None
                    }
                },
                Err(e) => {
                    eprintln!("  [conn ] {}: {}", m, e);
                    None
                }
            }
        }
    }))
    .buffer_unordered(cfg.concurrency)
    .collect()
    .await;

    // 4. 过滤成功项，按本次实测耗时升序
    let mut ok: Vec<(String, Duration)> = results.into_iter().flatten().collect();
    ok.sort_by_key(|(_, d)| *d);

    // [优化] 在 stderr 打印测速详情，让用户直观看到延迟差异
    if !ok.is_empty() {
        eprintln!("--- 测速结果 (按延迟升序) ---");
        for (m, d) in &ok {
            eprintln!("  ✓ {:<40} {}ms", m, d.as_millis());
        }
        eprintln!("----------------------------");
    }

    // 5. 输出。语义：
    //    - 传了 uri：输出 {mirror}/{uri} 列表，末尾追加原始 uri
    //    - 未传 uri：输出可用 mirror 本身
    let mut links: Vec<String> = match &uri {
        Some(_) => ok
            .into_iter()
            .map(|(m, _)| format!("{}/{}", m.trim_end_matches('/'), probe_path))
            .collect(),
        None => ok.into_iter().map(|(m, _)| m).collect(),
    };
    if let Some(u) = &uri {
        links.push(u.clone()); // 兜底原始链接
    }

    if links.is_empty() {
        eprintln!("所有镜像均不可达");
        return Ok(());
    }

    // 最终结果输出到 stdout，方便脚本管道提取
    println!("{}", links.join(" "));
    Ok(())
}

/// [新增] 规范化 probe 路径
/// 如果用户传入的是完整 URL (如 https://github.com/user/repo/blob/main/README.md)
/// 则提取出 `user/repo/blob/main/README.md`，避免与 mirror 域名错误拼接。
fn normalize_probe_path(raw: &str) -> String {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        // 简单的字符串分割提取 path，避免引入额外的 url crate 依赖
        if let Some(idx) = raw.find("://") {
            let after_scheme = &raw[idx + 3..];
            if let Some(path_idx) = after_scheme.find('/') {
                return after_scheme[path_idx + 1..].to_string();
            }
        }
    }
    raw.trim_start_matches('/').to_string()
}

/// 抓首页 HTML，找出 chunk URL，逐个下载并尝试抠 mirror 列表。
/// 第一个命中的 chunk 即为数据源；全部失败则用 fallback。
async fn fetch_mirrors(
    client: &reqwest::Client,
    cfg: &Config,
) -> Result<Vec<String>, Box<dyn Error>> {
    let chunk_re = Regex::new(&cfg.chunk_re)?;
    let mirror_re = Regex::new(&cfg.mirror_re)?;

    let mut candidates: Vec<String> = Vec::new();

    // [修复] 消除变量遮蔽 (shadowing)，使用不同名称区分 Response 和 String
    if let Ok(resp) = client.get(format!("{}/", cfg.site_origin)).send().await
        && let Ok(resp) = resp.error_for_status()
        && let Ok(html) = resp.text().await
    {
        for cap in chunk_re.captures_iter(&html) {
            let path = cap.get(0).unwrap().as_str();
            // chunk 路径可能是绝对 URL（带 CDN 域名）也可能是站内相对路径
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
