use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

const SYSTEM_PROMPT: &str = "You are the Globalwatch AI assistant. You help users understand global events, weather patterns, and crises displayed on the 3D globe. IMPORTANT: Keep responses to 1-2 short sentences only. Your responses are read aloud via text-to-speech, so brevity is essential. Do not think out loud or explain your reasoning. Just answer directly.";

pub async fn query_lm_studio(messages: Vec<ChatMessage>) -> Result<String, String> {
    let client = Client::new();

    let mut full_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];
    full_messages.extend(messages);

    let request = ChatRequest {
        model: "local-model".to_string(),
        messages: full_messages,
        temperature: 0.7,
        max_tokens: 150,
    };

    let resp = client
        .post("http://localhost:1234/v1/chat/completions")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("LM Studio request failed: {e}. Is LM Studio running?"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LM Studio returned {status}: {body}"));
    }

    let chat_resp: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse LM Studio response: {e}"))?;

    let raw = chat_resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LM Studio".to_string())?;

    // Strip <think>...</think> blocks from reasoning models
    Ok(strip_think_tags(&raw))
}

/// Remove <think>...</think> blocks that reasoning models (e.g. GLM, DeepSeek) emit.
fn strip_think_tags(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result.find("</think>") {
            result = format!("{}{}", &result[..start], &result[end + 8..]);
        } else {
            // Unclosed <think> — strip everything from <think> onward
            result = result[..start].to_string();
            break;
        }
    }
    result.trim().to_string()
}
