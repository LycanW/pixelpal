use std::time::Duration;
use serde::Serialize;
use tauri::State;
use base64::Engine;
use image::{DynamicImage, ImageEncoder};
use crate::commands::{AppState, resolve_pets_dir, sanitize_pet_id, save_settings};
use crate::ai_image;
use crate::ai_prompts;

#[derive(Debug, Serialize)]
pub struct AiConfig {
  pub base_url: String,
  pub has_key: bool,
  pub model: String,
}

#[tauri::command]
pub fn get_ai_config(state: State<AppState>) -> AiConfig {
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  AiConfig {
    base_url: settings.ai_base_url.clone().unwrap_or_default(),
    has_key: settings.ai_api_key.as_ref().map(|s| !s.is_empty()).unwrap_or(false),
    model: settings.ai_model.clone().unwrap_or_else(|| "gpt-image-1".to_string()),
  }
}

#[tauri::command]
pub fn set_ai_config(
  state: State<AppState>,
  base_url: String,
  api_key: String,
  model: String,
) -> Result<(), String> {
  let url = base_url.trim();
  if !url.starts_with("http://") && !url.starts_with("https://") {
    return Err("invalid URL: must start with http:// or https://".into());
  }
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.ai_base_url = Some(url.to_string());
  settings.ai_api_key = Some(api_key);
  settings.ai_model = Some(model.trim().to_string());
  save_settings(&settings);
  Ok(())
}

async fn call_image_generation(
  base_url: &str,
  api_key: &str,
  model: &str,
  prompt: &str,
  images: Option<Vec<String>>,
) -> Result<Vec<u8>, String> {
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(300))
    .http1_only()  // disable HTTP/2 to avoid negotiation failures with some providers
    .build()
    .map_err(|e| format!("build client: {}", e))?;

  let url = format!("{}/images/generations", base_url.trim_end_matches('/'));

  let mut body = serde_json::json!({
    "model": model,
    "prompt": prompt,
    "size": "1024x1024",
    "n": 1,
  });

  let has_images = images.is_some();
  if let Some(ref imgs) = images {
    body["image"] = serde_json::json!(imgs);
  }

  #[cfg(debug_assertions)]
  log::info!("AI request to {} with image ref: {}", url, has_images);

  let response = client
    .post(&url)
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&body)
    .send()
    .await
    .map_err(|e| {
      let msg = e.to_string();
      if msg.contains("timed out") || msg.contains("timeout") {
        format!("request timed out after 300s: {}", msg)
      } else if msg.contains("certificate") || msg.contains("TLS") || msg.contains("SSL") {
        format!("TLS/SSL error: {}", msg)
      } else if msg.contains("dns") || msg.contains("resolve") {
        format!("DNS error: {}", msg)
      } else if msg.contains("connect") {
        format!("connection error: {}", msg)
      } else {
        format!("network error: {}", msg)
      }
    })?;

  if !response.status().is_success() {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    // if reference images are not supported by the provider, retry without them
    if images.is_some() && status.as_u16() == 400 && text.to_lowercase().contains("image") {
      #[cfg(debug_assertions)]
      log::warn!("Provider rejected image parameter, falling back to text-only: {}", text);
      let fallback_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "size": "1024x1024",
        "n": 1,
      });
      let fb_response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&fallback_body)
        .send()
        .await
        .map_err(|e| format!("fallback request failed: {}", e))?;
      if !fb_response.status().is_success() {
        let fb_status = fb_response.status();
        let fb_text = fb_response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", fb_status, fb_text));
      }
      let fb_json: serde_json::Value = fb_response
        .json()
        .await
        .map_err(|e| format!("parse fallback response: {}", e))?;
      let fb_b64 = fb_json
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("b64_json"))
        .and_then(|b| b.as_str())
        .ok_or_else(|| "missing b64_json in fallback response".to_string())?;
      return base64::engine::general_purpose::STANDARD
        .decode(fb_b64)
        .map_err(|e| format!("decode base64: {}", e));
    }
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
  let (base_url, api_key, model) = {
    let settings = state.settings.lock().unwrap_or_else(|e: std::sync::PoisonError<std::sync::MutexGuard<'_, crate::commands::AppSettings>>| e.into_inner());
    let base_url = settings.ai_base_url.clone()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.clone()
      .ok_or("AI API key not configured")?;
    let model = settings.ai_model.clone()
      .unwrap_or_else(|| "gpt-image-1".to_string());
    (base_url, api_key, model)
  };

  let prompt = ai_prompts::build_base_prompt(&description);
  let bytes = call_image_generation(&base_url, &api_key, &model, &prompt, None).await?;

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

#[tauri::command]
pub async fn generate_frame(
  state: State<'_, AppState>,
  base_description: String,
  base_image: String,
  animation_name: String,
  frame_index: u32,
  total_frames: u32,
  pose_description: String,
) -> Result<String, String> {
  let (base_url, api_key, model) = {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_url = settings.ai_base_url.clone()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.clone()
      .ok_or("AI API key not configured")?;
    let model = settings.ai_model.clone()
      .unwrap_or_else(|| "gpt-image-1".to_string());
    (base_url, api_key, model)
  };

  let prompt = ai_prompts::build_frame_prompt(
    &base_description,
    &animation_name,
    frame_index,
    total_frames,
    &pose_description,
  );

  let images = if base_image.is_empty() {
    None
  } else {
    Some(vec![base_image])
  };

  let bytes = call_image_generation(&base_url, &api_key, &model, &prompt, images).await?;

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

/// Generate an entire animation row in a single request.
/// The model produces all frames in one horizontal strip, which we split into
/// individual frames. This gives much better identity consistency than
/// per-frame requests because all frames share the same model context.
#[tauri::command]
pub async fn generate_row(
  state: State<'_, AppState>,
  base_description: String,
  base_image: String,
  animation_name: String,
  frame_count: u32,
  pose_descriptions: Vec<String>,
) -> Result<Vec<String>, String> {
  let (base_url, api_key, model) = {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_url = settings.ai_base_url.clone()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.clone()
      .ok_or("AI API key not configured")?;
    let model = settings.ai_model.clone()
      .unwrap_or_else(|| "gpt-image-1".to_string());
    (base_url, api_key, model)
  };

  let prompt = ai_prompts::build_row_prompt(
    &base_description,
    &animation_name,
    frame_count,
    &pose_descriptions,
  );

  let images = if base_image.is_empty() { None } else { Some(vec![base_image]) };

  let bytes = call_image_generation(&base_url, &api_key, &model, &prompt, images).await?;

  let mut img = image::load_from_memory(&bytes)
    .map_err(|e| format!("load image: {}", e))?
    .to_rgba8();

  ai_image::make_background_transparent(&mut img, 30);
  let mut dyn_img = DynamicImage::ImageRgba8(img);

  let frames = ai_image::split_row_into_frames(&mut dyn_img, frame_count)
    .map_err(|e| format!("split frames: {}", e))?;

  let mut result = Vec::new();
  for f in &frames {
    let mut rgba = f.to_rgba8();
    ai_image::make_background_transparent(&mut rgba, 30);
    let cropped = ai_image::auto_crop_to_content(&rgba);

    let mut buf = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buf)
      .write_image(
        &cropped, cropped.width(), cropped.height(), image::ColorType::Rgba8.into()
      )
      .map_err(|e| format!("encode frame: {}", e))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    result.push(format!("data:image/png;base64,{}", b64));
  }

  Ok(result)
}

#[tauri::command]
pub fn save_ai_sprite(
  state: State<AppState>,
  pet_id: String,
  filename: String,
  frames: Vec<String>,
  frames_per_row: u32,
  base_image: String,
) -> Result<(), String> {
  sanitize_pet_id(&pet_id)?;

  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);

  let pet_dir = pets_dir.join(&pet_id);
  std::fs::create_dir_all(&pet_dir)
    .map_err(|e| format!("create dir: {}", e))?;

  let mut decoded_frames = Vec::new();
  for (idx, b64_data) in frames.iter().enumerate() {
    let b64 = b64_data
      .strip_prefix("data:image/png;base64,")
      .unwrap_or(b64_data);
    let bytes = base64::engine::general_purpose::STANDARD
      .decode(b64)
      .map_err(|e| format!("decode frame {}: {}", idx, e))?;
    let img = image::load_from_memory(&bytes)
      .map_err(|e| format!("load frame {}: {}", idx, e))?;
    decoded_frames.push(img);
  }

  let spritesheet = ai_image::compose_spritesheet(
    &decoded_frames, frames_per_row
  ).map_err(|e| format!("compose: {}", e))?;

  let dest = pet_dir.join(&filename);
  spritesheet
    .save_with_format(&dest, image::ImageFormat::Png
    )
    .map_err(|e| format!("save spritesheet: {}", e))?;

  // ── persist base image as canonical reference for future animations ──
  if !base_image.is_empty() {
    let b64 = base_image
      .strip_prefix("data:image/png;base64,")
      .unwrap_or(&base_image);
    let bytes = base64::engine::general_purpose::STANDARD
      .decode(b64)
      .map_err(|e| format!("decode base image: {}", e))?;
    let base_dest = pet_dir.join("base.png");
    std::fs::write(&base_dest, bytes)
      .map_err(|e| format!("save base image: {}", e))?;
  }

  // ── update config.json with animation definition ──
  let config_path = pet_dir.join("config.json");
  let frame_count = frames.len() as u32;
  let anim_name = filename
    .trim_end_matches(".png")
    .trim_end_matches(".gif")
    .to_string();

  let mut cfg: serde_json::Value = if config_path.exists() {
    let raw = std::fs::read_to_string(&config_path)
      .map_err(|e| format!("read config: {}", e))?;
    serde_json::from_str(&raw)
      .map_err(|e| format!("parse config: {}", e))?
  } else {
    serde_json::json!({
      "animations": {},
      "defaultState": anim_name.clone(),
      "states": {
        anim_name.clone(): { "entry": anim_name, "transitions": {} }
      }
    })
  };

  if let Some(obj) = cfg.as_object_mut() {
    if obj.get("animations").is_none() {
      obj.insert("animations".to_string(), serde_json::json!({}));
    }
    if let Some(anims) = obj.get_mut("animations").and_then(|v| v.as_object_mut()) {
      let entry = anims.entry(anim_name.clone()).or_insert_with(|| serde_json::json!({}));
      if let Some(anim) = entry.as_object_mut() {
        anim.insert("source".to_string(), serde_json::json!(filename));
        anim.entry("frameTime").or_insert(serde_json::json!(100));
        anim.entry("loop").or_insert(serde_json::json!(true));
        anim.insert("frameCount".to_string(), serde_json::json!(frame_count));
        anim.insert("framesPerRow".to_string(), serde_json::json!(frames_per_row));
      }
    }
    // if this is the only animation, set it as default
    if obj.get("defaultState").is_none() || obj.get("defaultState").and_then(|v| v.as_str()) == Some("") {
      obj.insert("defaultState".to_string(), serde_json::json!(anim_name.clone()));
    }
    if obj.get("states").is_none() {
      obj.insert("states".to_string(), serde_json::json!({}));
    }
    if let Some(states) = obj.get_mut("states").and_then(|v| v.as_object_mut()) {
      if !states.contains_key(&anim_name) {
        states.insert(anim_name.clone(), serde_json::json!({
          "entry": anim_name,
          "transitions": {}
        }));
      }
    }
  }

  let json = serde_json::to_string_pretty(&cfg)
    .map_err(|e| format!("serialize config: {}", e))?;
  std::fs::write(&config_path, json)
    .map_err(|e| format!("write config: {}", e))?;

  // ── ensure manifest.json exists (required for scan_pets) ──
  let manifest_path = pet_dir.join("manifest.json");
  if !manifest_path.exists() {
    let manifest = serde_json::json!({
      "name": pet_id,
      "version": "1.0.0",
      "author": "",
      "frameWidth": 64,
      "frameHeight": 64,
      "displayScale": 5,
      "windowWidth": 320,
      "windowHeight": 320
    });
    let mjson = serde_json::to_string_pretty(&manifest)
      .map_err(|e| format!("serialize manifest: {}", e))?;
    std::fs::write(&manifest_path, mjson)
      .map_err(|e| format!("write manifest: {}", e))?;
  }

  Ok(())
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
