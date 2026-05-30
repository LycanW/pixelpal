use std::time::Duration;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::commands::{AppState, save_settings};

#[derive(Debug, Serialize)]
pub struct AiConfig {
  pub base_url: String,
  pub has_key: bool,
}

#[tauri::command]
pub fn get_ai_config(state: State<AppState>) -> AiConfig {
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  AiConfig {
    base_url: settings.ai_base_url.clone().unwrap_or_default(),
    has_key: settings.ai_api_key.as_ref().map(|s| !s.is_empty()).unwrap_or(false),
  }
}

#[derive(Debug, Deserialize)]
pub struct SetAiConfigPayload {
  base_url: String,
  api_key: String,
}

#[tauri::command]
pub fn set_ai_config(state: State<AppState>, payload: SetAiConfigPayload) -> Result<(), String> {
  let url = payload.base_url.trim();
  if !url.starts_with("http://") && !url.starts_with("https://") {
    return Err("invalid URL: must start with http:// or https://".into());
  }
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.ai_base_url = Some(url.to_string());
  settings.ai_api_key = Some(payload.api_key);
  save_settings(&settings);
  Ok(())
}
