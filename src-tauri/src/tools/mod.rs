pub mod web_search;

use serde_json::Value;

/// Returns OpenAI-format tool definitions for all configured tools.
/// If no tools are configured (e.g. no API key set), returns an empty vec,
/// which causes llm.rs to fall back to the original no-tools behaviour.
pub fn definitions() -> Vec<Value> {
    let mut tools = Vec::new();

    if web_search::is_configured() {
        tools.push(serde_json::json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for current news, events, weather, or any topic. Use this when asked about recent or real-time information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        }
                    },
                    "required": ["query"]
                }
            }
        }));
    }

    tools
}

/// Execute a named tool with the given JSON arguments, returning a text result
/// that will be fed back into the conversation as a `tool` role message.
pub async fn execute(name: &str, args: &Value) -> Result<String, String> {
    match name {
        "web_search" => {
            let query = args["query"]
                .as_str()
                .ok_or_else(|| "web_search requires a 'query' argument".to_string())?;
            web_search::search(query).await
        }
        _ => Err(format!("Unknown tool: '{name}'")),
    }
}
