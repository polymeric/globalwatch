use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Public message type shared with the Tauri command layer.
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

// ── Request ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    /// Uses `Value` so we can mix plain `{role, content}` messages with the
    /// richer assistant tool-call and `role: "tool"` result shapes.
    messages: Vec<Value>,
    temperature: f32,
    max_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'static str>,
}

// ── Response ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct AssistantMessage {
    /// `None` when the model emits tool_calls instead of a text reply.
    content: Option<String>,
    tool_calls: Option<Vec<ToolCallResponse>>,
}

#[derive(Deserialize)]
struct ToolCallResponse {
    id: String,
    function: FunctionCall,
}

#[derive(Deserialize)]
struct FunctionCall {
    name: String,
    /// JSON-encoded arguments string, e.g. `{"query":"latest news"}`.
    arguments: String,
}

// ── System prompts ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = "\
You are the Globalwatch AI assistant. You help users understand global events, \
weather patterns, and crises displayed on the 3D globe. \
IMPORTANT: Keep responses to 1-2 short sentences only. \
Your responses are read aloud via text-to-speech, so brevity is essential. \
Do not think out loud or explain your reasoning. Just answer directly.";

const SYSTEM_PROMPT_TOOLS: &str = "\
You are the Globalwatch AI assistant. You help users understand global events, \
weather patterns, and crises displayed on the 3D globe. \
You have a web_search tool — use it to find current news, weather, or real-time \
information whenever the question requires up-to-date knowledge. \
After searching, summarise the key findings in 2-3 short sentences. \
Responses are read aloud via TTS so be concise. Do not think out loud.";

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn query_lm_studio(messages: Vec<ChatMessage>) -> Result<String, String> {
    let client = Client::new();

    // Only attach tools when at least one is configured (e.g. Kagi key present).
    let tools = crate::tools::definitions();
    let use_tools = !tools.is_empty();

    let system_prompt = if use_tools {
        SYSTEM_PROMPT_TOOLS
    } else {
        SYSTEM_PROMPT
    };

    // Seed the conversation with the system message followed by the caller's history.
    let mut conversation: Vec<Value> = std::iter::once(serde_json::json!({
        "role": "system",
        "content": system_prompt,
    }))
    .chain(messages.iter().map(|m| serde_json::json!({
        "role": m.role,
        "content": m.content,
    })))
    .collect();

    // Tool-calling loop — iterate until the model returns a text response.
    // Capped at 5 rounds to prevent runaway tool chains.
    for iteration in 0..5u8 {
        let request = ChatRequest {
            model: "local-model".to_string(),
            messages: conversation.clone(),
            temperature: 0.7,
            // Give the model more room when it may need to process search results.
            max_tokens: if use_tools { 500 } else { 150 },
            tools: if use_tools { Some(tools.clone()) } else { None },
            tool_choice: if use_tools { Some("auto") } else { None },
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

        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| "No response from LM Studio".to_string())?;

        match choice.finish_reason.as_deref() {
            Some("tool_calls") => {
                let tool_calls = choice.message.tool_calls.unwrap_or_default();
                if tool_calls.is_empty() {
                    break;
                }

                log::info!(
                    "LLM requested {} tool call(s) on iteration {iteration}",
                    tool_calls.len()
                );

                // Append the assistant's tool-call message so the model can see
                // what it asked for when we feed results back.
                conversation.push(serde_json::json!({
                    "role": "assistant",
                    "content": null,
                    "tool_calls": tool_calls.iter().map(|tc| serde_json::json!({
                        "id": tc.id,
                        "type": "function",
                        "function": {
                            "name": tc.function.name,
                            "arguments": tc.function.arguments,
                        }
                    })).collect::<Vec<_>>()
                }));

                // Execute each tool and append its result.
                for tc in &tool_calls {
                    let args: Value = serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(Value::Object(Default::default()));

                    log::info!(
                        "Executing tool '{}' with args: {}",
                        tc.function.name,
                        tc.function.arguments
                    );

                    let result = crate::tools::execute(&tc.function.name, &args)
                        .await
                        .unwrap_or_else(|e| format!("Tool error: {e}"));

                    log::info!(
                        "Tool '{}' returned {} chars",
                        tc.function.name,
                        result.len()
                    );

                    conversation.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tc.id,
                        "content": result,
                    }));
                }
                // Loop continues — model will synthesise a reply from the results.
            }

            _ => {
                // Normal text response (finish_reason "stop" or anything else).
                let raw = choice
                    .message
                    .content
                    .ok_or_else(|| "Empty response from LM Studio".to_string())?;
                return Ok(strip_think_tags(&raw));
            }
        }
    }

    Err("LLM did not produce a text response after tool calls".to_string())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Remove `<think>…</think>` blocks emitted by reasoning models (DeepSeek, GLM, etc.)
fn strip_think_tags(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result.find("</think>") {
            result = format!("{}{}", &result[..start], &result[end + 8..]);
        } else {
            // Unclosed <think> — strip everything from it onward.
            result = result[..start].to_string();
            break;
        }
    }
    result.trim().to_string()
}
