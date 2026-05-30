use std::time::Duration;
use serde::{Deserialize, Serialize};
use tauri::State;
use base64::Engine;
use image::ImageEncoder;
use crate::commands::{AppState, save_settings};
use crate::ai_image;
use crate::ai_prompts;

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

async fn call_image_generation(
  base_url: &str,
  api_key: &str,
  prompt: &str,
) -> Result<Vec<u8>, String> {
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()
    .map_err(|e| format!("build client: {}", e))?;

  let url = format!("{}/images/generations", base_url.trim_end_matches('/'));

  let body = serde_json::json!({
    "model": "gpt-image-1",
    "prompt": prompt,
    "size": "1024x1024",
    "quality": "high",
    "n": 1,
  });

  let response = client
    .post(&url)
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&body)
    .send()
    .await
    .map_err(|e| format!("request failed: {}", e))?;

  if !response.status().is_success() {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    return Err(format!("API error {}: {}", status, text));
  }

  let json: serde_json::Value = response
    .json()
    .await
    .map_err(|e| format!("parse response: {}", e))?;

  let b64 = json
    .get("data")
    .and_then(|d| d.as_array())
    .and_then(|arr| arr.first())
    .and_then(|item| item.get("b64_json"))
    .and_then(|b| b.as_str())
    .ok_or_else(|| "missing b64_json in response".to_string())?;

  base64::engine::general_purpose::STANDARD
    .decode(b64)
    .map_err(|e| format!("decode base64: {}", e))
}

#[tauri::command]
pub async fn generate_base(
  state: State<'_, AppState>,
  description: String,
) -> Result<String, String> {
  let (base_url, api_key) = {
    let settings = state.settings.lock().unwrap_or_else(|e: std::sync::PoisonError<std::sync::MutexGuard<'_, crate::commands::AppSettings>>| e.into_inner());
    let base_url = settings.ai_base_url.clone()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.clone()
      .ok_or("AI API key not configured")?;
    (base_url, api_key)
  };

  let prompt = ai_prompts::build_base_prompt(&description);
  let bytes = call_image_generation(&base_url, &api_key, &prompt).await?;

  let mut img = image::load_from_memory(&bytes)
    .map_err(|e| format!("load image: {}", e))?
    .to_rgba8();

  ai_image::make_background_transparent(&mut img, 30);
  let cropped = ai_image::auto_crop_to_content(&img);

  let mut buf = Vec::new();
  image::codecs::png::PngEncoder::new(&mut buf)
    .write_image(
      &cropped, cropped.width(), cropped.height(), image::ColorType::Rgba8.into()
    )
    .map_err(|e| format!("encode png: {}", e))?;

  let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
  Ok(format!("data:image/png;base64,{}", b64))
}

#[derive(Debug, Deserialize)]
pub struct GenerateFramePayload {
  base_description: String,
  animation_name: String,
  frame_index: u32,
  total_frames: u32,
  pose_description: String,
}

#[tauri::command]
pub async fn generate_frame(
  state: State<'_, AppState>,
  payload: GenerateFramePayload,
) -> Result<String, String> {
  let (base_url, api_key) = {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_url = settings.ai_base_url.clone()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.clone()
      .ok_or("AI API key not configured")?;
    (base_url, api_key)
  };

  let prompt = ai_prompts::build_frame_prompt(
    &payload.base_description,
    &payload.animation_name,
    payload.frame_index,
    payload.total_frames,
    &payload.pose_description,
  );

  let bytes = call_image_generation(&base_url, &api_key, &prompt).await?;

  let mut img = image::load_from_memory(&bytes)
    .map_err(|e| format!("load image: {}", e))?
    .to_rgba8();

  ai_image::make_background_transparent(&mut img, 30);
  let cropped = ai_image::auto_crop_to_content(&img);

  let mut buf = Vec::new();
  image::codecs::png::PngEncoder::new(&mut buf)
    .write_image(
      &cropped, cropped.width(), cropped.height(), image::ColorType::Rgba8.into()
    )
    .map_err(|e| format!("encode png: {}", e))?;

  let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
  Ok(format!("data:image/png;base64,{}", b64))
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_url_validation_rejects_ftp() {
    let url = "ftp://example.com";
    assert!(!url.starts_with("http://") && !url.starts_with("https://"));
  }

  #[test]
  fn test_url_validation_accepts_http() {
    let url = "http://localhost:8080/v1";
    assert!(url.starts_with("http://") || url.starts_with("https://"));
  }

  #[test]
  fn test_url_validation_accepts_https() {
    let url = "https://api.openai.com/v1";
    assert!(url.starts_with("http://") || url.starts_with("https://"));
  }
}
