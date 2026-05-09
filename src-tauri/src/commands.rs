use std::path::{Path, PathBuf};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
  pub pets_dir: Option<String>,
  pub active_pet: Option<String>,
  pub always_on_top: Option<bool>,
  pub scale: Option<u32>,
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      pets_dir: None,
      active_pet: Some("default-cat".into()),
      always_on_top: Some(true),
      scale: Some(5),
    }
  }
}

pub struct AppState {
  pub settings: std::sync::Mutex<AppSettings>,
}

pub fn settings_path() -> PathBuf {
  dirs::data_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("pixelpal")
    .join("app-settings.json")
}

pub fn load_settings() -> AppSettings {
  let path = settings_path();
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if path.exists() {
    std::fs::read_to_string(&path)
      .ok()
      .and_then(|s| serde_json::from_str(&s).ok())
      .unwrap_or_default()
  } else {
    AppSettings::default()
  }
}

pub fn save_settings(settings: &AppSettings) {
  let path = settings_path();
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Ok(json) = serde_json::to_string_pretty(settings) {
    let _ = std::fs::write(&path, json);
  }
}

pub fn resolve_pets_dir(settings: &AppSettings) -> PathBuf {
  if let Some(custom) = &settings.pets_dir {
    let p = PathBuf::from(custom);
    if p.exists() {
      return p;
    }
  }
  let exe = std::env::current_exe().ok();
  if let Some(exe_path) = exe {
    let parent = exe_path.parent().unwrap();
    let mut dir = parent.to_path_buf();
    for _ in 0..6 {
      let candidate = dir.join("pets");
      if candidate.exists() {
        return candidate;
      }
      if let Some(p) = dir.parent() {
        dir = p.to_path_buf();
      } else {
        break;
      }
    }
    parent.join("pets")
  } else {
    PathBuf::from("pets")
  }
}

pub fn scan_pets(pets_dir: &Path) -> Vec<String> {
  let mut pets = Vec::new();
  if !pets_dir.exists() {
    return pets;
  }
  if let Ok(entries) = std::fs::read_dir(pets_dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() && path.join("manifest.json").exists() {
        if let Some(name) = path.file_name() {
          pets.push(name.to_string_lossy().to_string());
        }
      }
    }
  }
  pets.sort();
  pets
}

#[tauri::command]
pub fn list_pets(state: State<AppState>) -> Vec<String> {
  let settings = state.settings.lock().unwrap();
  let dir = resolve_pets_dir(&settings);
  scan_pets(&dir)
}

fn sanitize_pet_id(id: &str) -> Result<(), String> {
  if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
    return Err("invalid pet id".into());
  }
  Ok(())
}

fn sanitize_pet_path(pets_dir: &Path, id: &str, filename: &str) -> Result<PathBuf, String> {
  // Prevent path traversal — reject ids/filenames with parent dir components
  if id.contains("..") || id.contains('/') || id.contains('\\') {
    return Err("invalid pet id".into());
  }
  if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
    return Err("invalid filename".into());
  }
  let path = pets_dir.join(id).join(filename);
  // Canonicalize the pets_dir to resolve symlinks, then verify the resolved path is under it
  let canonical_root = std::fs::canonicalize(pets_dir)
    .map_err(|_| "pets directory not found".to_string())?;
  // For the target, try canonicalize (file exists) or manually resolve from components
  let canonical_target = match std::fs::canonicalize(&path) {
    Ok(p) => p,
    Err(_) => {
      // File may not exist yet; resolve the parent directory and append filename
      let parent = path.parent().unwrap();
      let file_name = path.file_name().unwrap();
      let canonical_parent = std::fs::canonicalize(parent)
        .map_err(|_| format!("directory not found: {}", parent.display()))?;
      canonical_parent.join(file_name)
    }
  };
  if !canonical_target.starts_with(&canonical_root) {
    return Err("path traversal detected".into());
  }
  Ok(path)
}

#[tauri::command]
pub fn read_json(state: State<AppState>, id: String, filename: String) -> Result<String, String> {
  let settings = state.settings.lock().unwrap();
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))
}

#[tauri::command]
pub fn write_json(state: State<AppState>, id: String, filename: String, content: String) -> Result<(), String> {
  let settings = state.settings.lock().unwrap();
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::write(&path, &content).map_err(|e| format!("write {}: {}", path.display(), e))
}

#[tauri::command]
pub fn read_pet_sprite(state: State<AppState>, id: String, filename: String) -> Result<String, String> {
  let settings = state.settings.lock().unwrap();
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  let data = std::fs::read(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
  let ext = Path::new(&filename).extension().and_then(|s| s.to_str()).unwrap_or("png");
  let mime = match ext {
    "webp" => "image/webp",
    "jpg" | "jpeg" => "image/jpeg",
    "gif" => "image/gif",
    _ => "image/png",
  };
  let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
  Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
pub fn get_pets_dir(state: State<AppState>) -> String {
  let settings = state.settings.lock().unwrap();
  resolve_pets_dir(&settings).to_string_lossy().to_string()
}

#[tauri::command]
pub fn set_pets_dir(state: State<AppState>, path: String) -> Result<(), String> {
  let p = PathBuf::from(&path);
  if !p.exists() {
    return Err("directory not found".into());
  }
  let mut settings = state.settings.lock().unwrap();
  settings.pets_dir = Some(path);
  save_settings(&settings);
  Ok(())
}

#[tauri::command]
pub fn get_active_pet(state: State<AppState>) -> String {
  let settings = state.settings.lock().unwrap();
  settings.active_pet.clone().unwrap_or_else(|| "default-cat".into())
}

#[tauri::command]
pub fn set_active_pet(state: State<AppState>, id: String) -> Result<(), String> {
  sanitize_pet_id(&id)?;
  let mut settings = state.settings.lock().unwrap();
  settings.active_pet = Some(id);
  save_settings(&settings);
  Ok(())
}

#[tauri::command]
pub fn toggle_window(app: tauri::AppHandle) {
  if let Some(window) = app.get_webview_window("main") {
    if window.is_visible().unwrap_or(false) {
      let _ = window.hide();
    } else {
      let _ = window.show();
      let _ = window.set_focus();
    }
  }
}

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) {
  if let Some(window) = app.get_webview_window("settings") {
    let _ = window.set_focus();
    return;
  }
  let _ = tauri::WebviewWindowBuilder::new(
    &app,
    "settings",
    tauri::WebviewUrl::App("settings.html".into()),
  )
  .title("Settings")
  .inner_size(620.0, 540.0)
  .center()
  .build();
}

#[tauri::command]
pub fn set_always_on_top(app: tauri::AppHandle, state: State<AppState>, on: bool) {
  if let Some(window) = app.get_webview_window("main") {
    let _ = window.set_always_on_top(on);
  }
  let mut settings = state.settings.lock().unwrap();
  settings.always_on_top = Some(on);
  save_settings(&settings);
}

#[tauri::command]
pub fn get_always_on_top(app: tauri::AppHandle) -> bool {
  app.get_webview_window("main")
    .and_then(|w| w.is_always_on_top().ok())
    .unwrap_or(true)
}

#[tauri::command]
pub fn get_scale(state: tauri::State<AppState>) -> u32 {
  let settings = state.settings.lock().unwrap();
  settings.scale.unwrap_or(5)
}

#[tauri::command]
pub fn set_scale(state: tauri::State<AppState>, scale: u32) {
  let mut settings = state.settings.lock().unwrap();
  settings.scale = Some(scale.clamp(1, 10));
  save_settings(&settings);
}

#[tauri::command]
pub fn create_pet(state: tauri::State<AppState>, name: String, frame_size: u32, display_scale: u32) -> Result<(), String> {
  sanitize_pet_id(&name)?;
  let settings = state.settings.lock().unwrap();
  let dir = resolve_pets_dir(&settings).join(&name);
  drop(settings);
  std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {}", e))?;
  let manifest = serde_json::json!({
    "name": name,
    "version": "1.0.0",
    "author": "",
    "frameWidth": frame_size,
    "frameHeight": frame_size,
    "displayScale": display_scale,
    "windowWidth": frame_size * display_scale,
    "windowHeight": frame_size * display_scale,
  });
  std::fs::write(
    dir.join("manifest.json"),
    serde_json::to_string_pretty(&manifest).unwrap(),
  ).map_err(|e| format!("write manifest: {}", e))?;
  let config = serde_json::json!({
    "animations": {
      "idle": {"source": "idle.png", "frameTime": 600, "loop": true}
    },
    "defaultState": "idle",
    "states": {
      "idle": {
        "entry": "idle",
        "transitions": {}
      }
    }
  });
  std::fs::write(
    dir.join("config.json"),
    serde_json::to_string_pretty(&config).unwrap(),
  ).map_err(|e| format!("write config: {}", e))?;
  Ok(())
}

#[tauri::command]
pub fn list_pet_images(state: tauri::State<AppState>, id: String) -> Result<Vec<String>, String> {
  sanitize_pet_id(&id)?;
  let settings = state.settings.lock().unwrap();
  let dir = resolve_pets_dir(&settings).join(&id);
  drop(settings);
  let mut images = Vec::new();
  if let Ok(entries) = std::fs::read_dir(&dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_file() {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
          if matches!(ext.to_lowercase().as_str(), "png" | "webp" | "jpg" | "jpeg" | "gif") {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
              images.push(name.to_string());
            }
          }
        }
      }
    }
  }
  images.sort();
  Ok(images)
}

#[tauri::command]
pub fn delete_pet_image(state: tauri::State<AppState>, id: String, filename: String) -> Result<(), String> {
  sanitize_pet_id(&id)?;
  let ext = Path::new(&filename).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
  if !matches!(ext.as_str(), "png" | "webp" | "jpg" | "jpeg" | "gif") {
    return Err("not an image file".into());
  }
  let settings = state.settings.lock().unwrap();
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::remove_file(&path).map_err(|e| format!("delete {}: {}", path.display(), e))
}

#[tauri::command]
pub async fn import_pet(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
  use tauri_plugin_dialog::DialogExt;
  let path = app.dialog()
    .file()
    .blocking_pick_folder();
  if let Some(p) = path {
    let src = p.as_path().unwrap();
    let name = src.file_name().unwrap().to_string_lossy().to_string();
    let settings = state.settings.lock().unwrap();
    let dest = resolve_pets_dir(&settings).join(&name);
    drop(settings);
    if !dest.exists() {
      copy_dir_all(src, &dest).map_err(|e| format!("copy: {}", e))?;
    }
  }
  Ok(())
}

#[tauri::command]
pub async fn import_pet_image(app: tauri::AppHandle, state: tauri::State<'_, AppState>, id: String) -> Result<String, String> {
  sanitize_pet_id(&id)?;
  use tauri_plugin_dialog::DialogExt;
  let path = app.dialog()
    .file()
    .add_filter("Images", &["png", "webp", "jpg", "jpeg", "gif"])
    .blocking_pick_file();
  if let Some(p) = path {
    let src = p.as_path().unwrap();
    let fname = src.file_name().unwrap().to_string_lossy().to_string();
    let settings = state.settings.lock().unwrap();
    let dir = resolve_pets_dir(&settings).join(&id);
    drop(settings);
    let dest = dir.join(&fname);
    std::fs::copy(src, &dest).map_err(|e| format!("copy: {}", e))?;
    Ok(fname.to_string())
  } else {
    Err("No file selected".into())
  }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
  std::fs::create_dir_all(dst)?;
  for entry in std::fs::read_dir(src)? {
    let entry = entry?;
    let ty = entry.file_type()?;
    if ty.is_dir() {
      copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
    } else {
      std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
    }
  }
  Ok(())
}
