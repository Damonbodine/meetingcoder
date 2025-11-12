use crate::summarization::llm;
use tauri::command;

#[command]
pub async fn store_claude_api_key(api_key: String) -> Result<(), String> {
    llm::store_api_key(&api_key).map_err(|e| e.to_string())
}

#[command]
pub async fn has_claude_api_key() -> Result<bool, String> {
    Ok(llm::has_api_key())
}

#[command]
pub async fn delete_claude_api_key() -> Result<(), String> {
    llm::delete_api_key().map_err(|e| e.to_string())
}
