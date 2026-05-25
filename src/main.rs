use serde::{ Deserialize, Serialize };
use reqwest::header;
use std::env;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ApiResponse {
    code: i32,
    msg: String,
    data: Option<Vec<Mirror>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Mirror {
    url: String,
    #[serde(default)]
    latency: i32,
    #[serde(default)]
    speed: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    let test = "https://raw.githubusercontent.com/hglcode/xshrc/refs/heads/main/README.md";
    let uri = args.get(1).map(|s| s.clone());

    // 创建客户端
    let client = reqwest::Client
        ::builder()
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
            headers
        })
        .timeout(Duration::from_secs(10))
        .build()?;

    // 调用 API
    let response = client.get("https://api.akams.cn/github").send().await?;
    let api_response: ApiResponse = response.json().await?;

    if api_response.code != 200 {
        eprintln!("API error: {}", api_response.msg);
        return Ok(());
    }

    // 排序数据
    let mut mirrors = api_response.data.unwrap_or_default();
    mirrors.sort_by(|a, b| {
        b.speed
            .partial_cmp(&a.speed)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.latency.cmp(&b.latency))
    });

    // 测试所有镜像
    let test_futures: Vec<_> = mirrors
        .iter()
        .map(|mirror| {
            let client = client.clone();
            let mirror = mirror.url.clone();
            let test = test.to_string();
            let uri = uri.clone();

            async move {
                let url = format!("{}/{}", mirror, test);

                match client.get(&url).send().await {
                    Ok(_) => {
                        if let Some(u) = uri {
                            Some(format!("{}/{}", mirror, u))
                        } else {
                            Some(mirror)
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching {}: {:?}", mirror, e);
                        None
                    }
                }
            }
        })
        .collect();

    // 并发执行所有测试
    let results = futures::future::join_all(test_futures).await;

    // 过滤并输出
    let mut links: Vec<String> = results
        .into_iter()
        .filter_map(|x| x)
        .collect();

    if let Some(u) = &uri {
        links.push(u.clone());
    }

    println!("{}", links.join(" "));

    Ok(())
}
