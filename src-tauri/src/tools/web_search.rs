use reqwest::Client;
use serde::Deserialize;

const KAGI_SEARCH_URL: &str = "https://kagi.com/api/v0/search";

#[derive(Deserialize)]
struct KagiResponse {
    data: Option<Vec<KagiResult>>,
}

#[derive(Deserialize)]
struct KagiResult {
    /// t=0: search result, t=1: related queries — we only use t=0
    t: u8,
    url: Option<String>,
    title: Option<String>,
    snippet: Option<String>,
}

/// Returns true if a Kagi API key is available, meaning the web_search tool
/// can be offered to the model.
pub fn is_configured() -> bool {
    load_api_key().is_ok()
}

pub async fn search(query: &str) -> Result<String, String> {
    let api_key = load_api_key()?;

    let client = Client::new();
    let resp = client
        .get(KAGI_SEARCH_URL)
        .header("Authorization", format!("Bot {api_key}"))
        .query(&[("q", query), ("limit", "5")])
        .send()
        .await
        .map_err(|e| format!("Kagi Search request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Kagi Search returned {status}: {body}"));
    }

    let kagi_resp: KagiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Kagi Search response: {e}"))?;

    let results: Vec<String> = kagi_resp
        .data
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.t == 0) // t=0 are search results; t=1 are related queries
        .take(5)
        .map(|r| {
            let title = r.title.as_deref().unwrap_or("Untitled");
            let url = r.url.as_deref().unwrap_or("");
            let snippet = r.snippet.as_deref().unwrap_or("No description available.");
            format!("[{title}]\n{url}\n{snippet}")
        })
        .collect();

    if results.is_empty() {
        return Ok("No search results found.".to_string());
    }

    Ok(results.join("\n\n"))
}

/// Resolves the Kagi API key from (in order of preference):
/// 1. `KAGI_API_KEY` environment variable
/// 2. `sources.config.json` — entry with id "kagi-search", field "token"
///    Checked in: current dir, parent dir (src-tauri/ dev layout), exe dir
fn load_api_key() -> Result<String, String> {
    // 1. Environment variable
    if let Ok(key) = std::env::var("KAGI_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    // 2. sources.config.json — check several candidate directories
    let candidates: Vec<std::path::PathBuf> = [
        std::env::current_dir().ok(),
        std::env::current_dir().ok().map(|d| d.join("..")),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf())),
    ]
    .into_iter()
    .flatten()
    .collect();

    for dir in candidates {
        let path = dir.join("sources.config.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(sources) = config["sources"].as_array() {
                    for source in sources {
                        if source["id"].as_str() == Some("kagi-search") {
                            if let Some(token) = source["token"].as_str() {
                                if !token.is_empty() {
                                    return Ok(token.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err("Kagi API key not configured. Set KAGI_API_KEY env var or add token to sources.config.json under id \"kagi-search\".".to_string())
}
