use std::path::{Path, PathBuf};
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
  pub pets_dir: Option<String>,
  pub active_pet: Option<String>,
  pub always_on_top: Option<bool>,
  pub scale: Option<u32>,
  pub language: Option<String>,
  pub autostart: Option<bool>,
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      pets_dir: None,
      active_pet: Some("default-cat".into()),
      always_on_top: Some(true),
      scale: Some(5),
      language: Some("zh".into()),
      autostart: Some(false),
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
    if let Ok(raw) = std::fs::read_to_string(&path) {
      match serde_json::from_str(&raw) {
        Ok(settings) => return settings,
        Err(e) => log::warn!("failed to parse settings, using defaults: {}", e),
      }
    }
    AppSettings::default()
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
  dirs::data_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("pixelpal")
    .join("pets")
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::read_to_string(&path).map_err(|e| format!("read {}/{}: {}", id, filename, e))
}

#[tauri::command]
pub fn write_json(state: State<AppState>, id: String, filename: String, content: String) -> Result<(), String> {
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::write(&path, &content).map_err(|e| format!("write {}/{}: {}", id, filename, e))
}

#[tauri::command]
pub fn read_pet_sprite(state: State<AppState>, id: String, filename: String) -> Result<String, String> {
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  const MAX_SIZE: u64 = 10 * 1024 * 1024;
  let meta = std::fs::metadata(&path).map_err(|e| format!("stat {}/{}: {}", id, filename, e))?;
  if meta.len() > MAX_SIZE {
    return Err(format!("sprite file too large ({} bytes, max {} bytes)", meta.len(), MAX_SIZE));
  }
  let data = std::fs::read(&path).map_err(|e| format!("read {}/{}: {}", id, filename, e))?;
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  resolve_pets_dir(&settings).to_string_lossy().to_string()
}

#[tauri::command]
pub fn set_pets_dir(state: State<AppState>, path: String) -> Result<(), String> {
  let p = PathBuf::from(&path);
  if !p.exists() {
    return Err("directory not found".into());
  }
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.pets_dir = Some(path);
  save_settings(&settings);
  Ok(())
}

pub fn resolve_active_pet(active_pet: Option<String>, pets_dir: &Path) -> String {
  let active = active_pet.unwrap_or_else(|| "default-cat".into());
  let pets = scan_pets(pets_dir);
  if pets.is_empty() {
    return "".into();
  }
  if pets.contains(&active) {
    active
  } else {
    pets.into_iter().next().unwrap_or(active)
  }
}

#[tauri::command]
pub fn get_active_pet(state: State<AppState>) -> String {
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let active = settings.active_pet.clone();
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  resolve_active_pet(active, &pets_dir)
}

#[tauri::command]
pub fn set_active_pet(state: State<AppState>, id: String) -> Result<(), String> {
  sanitize_pet_id(&id)?;
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.scale.unwrap_or(5)
}

#[tauri::command]
pub fn set_scale(app: tauri::AppHandle, state: tauri::State<AppState>, scale: u32) {
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.scale = Some(scale.clamp(1, 10));
  save_settings(&settings);
  let _ = app.emit("scale-changed", scale);
}

#[tauri::command]
pub fn get_language(state: tauri::State<AppState>) -> String {
  state.settings.lock().unwrap_or_else(|e| e.into_inner()).language.clone().unwrap_or_else(|| "zh".into())
}

#[tauri::command]
pub fn set_language(state: tauri::State<AppState>, lang: String) {
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.language = Some(lang);
  save_settings(&settings);
}

#[tauri::command]
pub fn get_autostart(state: tauri::State<AppState>) -> bool {
  state.settings.lock().unwrap_or_else(|e| e.into_inner()).autostart.unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, state: tauri::State<AppState>, on: bool) {
  let mut settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  settings.autostart = Some(on);
  save_settings(&settings);
  drop(settings);
  let _ = if on {
    app.autolaunch().enable()
  } else {
    app.autolaunch().disable()
  };
}

#[tauri::command]
pub fn create_pet(state: tauri::State<AppState>, name: String, _frame_size: u32, _display_scale: u32) -> Result<(), String> {
  sanitize_pet_id(&name)?;
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let dir = resolve_pets_dir(&settings).join(&name);
  drop(settings);
  std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {}", e))?;
  let manifest = serde_json::json!({
    "name": name,
    "version": "1.0.0",
    "author": "",
  });
  std::fs::write(
    dir.join("manifest.json"),
    serde_json::to_string_pretty(&manifest).unwrap(),
  ).map_err(|e| format!("write manifest: {}", e))?;
  let config = serde_json::json!({
    "animations": {},
    "defaultState": "",
    "states": {}
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
  let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
  let pets_dir = resolve_pets_dir(&settings);
  drop(settings);
  // Check if file is still referenced in config.json
  let config_path = sanitize_pet_path(&pets_dir, &id, "config.json")?;
  if let Ok(raw) = std::fs::read_to_string(&config_path) {
    if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&raw) {
      for (_, anim) in cfg.get("animations").and_then(|a| a.as_object()).into_iter().flatten() {
        if anim.get("source").and_then(|s| s.as_str()) == Some(&filename) {
          return Err("image is still referenced by an animation".into());
        }
      }
    }
  }
  let path = sanitize_pet_path(&pets_dir, &id, &filename)?;
  std::fs::remove_file(&path).map_err(|e| format!("delete {}/{}: {}", id, filename, e))
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
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
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
  if src.is_symlink() {
    return Ok(());
  }
  std::fs::create_dir_all(dst)?;
  for entry in std::fs::read_dir(src)? {
    let entry = entry?;
    if entry.path().is_symlink() {
      continue;
    }
    let ty = entry.file_type()?;
    if ty.is_dir() {
      copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
    } else {
      std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
    }
  }
  Ok(())
}


#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  // ── sanitize_pet_id ──

  #[test]
  fn test_sanitize_pet_id_valid() {
    assert!(sanitize_pet_id("default-cat").is_ok());
    assert!(sanitize_pet_id("MP").is_ok());
    assert!(sanitize_pet_id("1234").is_ok());
  }

  #[test]
  fn test_sanitize_pet_id_empty() {
    assert!(sanitize_pet_id("").is_err());
  }

  #[test]
  fn test_sanitize_pet_id_dotdot() {
    assert!(sanitize_pet_id("..").is_err());
    assert!(sanitize_pet_id("foo..bar").is_err());
  }

  #[test]
  fn test_sanitize_pet_id_slash() {
    assert!(sanitize_pet_id("foo/bar").is_err());
  }

  #[test]
  fn test_sanitize_pet_id_backslash() {
    assert!(sanitize_pet_id("foo\\bar").is_err());
  }

  // ── sanitize_pet_path ──

  #[test]
  fn test_sanitize_pet_path_valid_file() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("cat")).unwrap();
    fs::write(pets_dir.join("cat").join("idle.png"), "fake").unwrap();

    let result = sanitize_pet_path(&pets_dir, "cat", "idle.png");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), pets_dir.join("cat").join("idle.png"));
  }

  #[test]
  fn test_sanitize_pet_path_nonexistent_file_in_valid_dir() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("cat")).unwrap();

    let result = sanitize_pet_path(&pets_dir, "cat", "missing.png");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), pets_dir.join("cat").join("missing.png"));
  }

  #[test]
  fn test_sanitize_pet_path_dotdot_in_id() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    assert!(sanitize_pet_path(&pets_dir, "..", "x.png").is_err());
  }

  #[test]
  fn test_sanitize_pet_path_dotdot_in_filename() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("cat")).unwrap();

    assert!(sanitize_pet_path(&pets_dir, "cat", "../x.png").is_err());
  }

  #[test]
  fn test_sanitize_pet_path_traversal_via_filename() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("cat")).unwrap();

    assert!(sanitize_pet_path(&pets_dir, "cat", "../../etc/passwd").is_err());
  }

  #[test]
  fn test_sanitize_pet_path_missing_pets_dir() {
    let result = sanitize_pet_path(Path::new("/nonexistent/path"), "cat", "idle.png");
    assert!(result.is_err());
  }

  #[test]
  fn test_sanitize_pet_path_traversal_resolved() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("cat")).unwrap();
    // Create a symlink or adjacent dir to test canonical path boundary
    fs::create_dir(dir.path().join("outside")).unwrap();
    #[cfg(unix)]
    {
      use std::os::unix::fs::symlink;
      let _ = symlink(dir.path().join("outside"), pets_dir.join("cat").join("link"));
    }

    // Even without symlink, filename with .. should be rejected before canonicalization
    assert!(sanitize_pet_path(&pets_dir, "cat", "link/../../outside.txt").is_err());
  }

  // ── resolve_pets_dir ──

  #[test]
  fn test_resolve_pets_dir_custom_exists() {
    let dir = TempDir::new().unwrap();
    let custom = dir.path().join("my_pets");
    fs::create_dir(&custom).unwrap();

    let settings = AppSettings {
      pets_dir: Some(custom.to_string_lossy().to_string()),
      ..Default::default()
    };

    assert_eq!(resolve_pets_dir(&settings), custom);
  }

  #[test]
  fn test_resolve_pets_dir_custom_not_exists_fallback() {
    let settings = AppSettings {
      pets_dir: Some("/nonexistent/path".into()),
      ..Default::default()
    };

    let result = resolve_pets_dir(&settings);
    let s = result.to_string_lossy();
    assert!(s.contains("pixelpal"), "fallback should contain pixelpal: {}", s);
    assert!(s.contains("pets"), "fallback should contain pets: {}", s);
  }

  #[test]
  fn test_resolve_pets_dir_none_fallback() {
    let settings = AppSettings::default();
    let result = resolve_pets_dir(&settings);
    let s = result.to_string_lossy();
    assert!(s.contains("pixelpal"), "fallback should contain pixelpal: {}", s);
    assert!(s.contains("pets"), "fallback should contain pets: {}", s);
  }

  // ── scan_pets ──

  #[test]
  fn test_scan_pets_empty() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    assert!(scan_pets(&pets_dir).is_empty());
  }

  #[test]
  fn test_scan_pets_skips_missing_manifest() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::create_dir(pets_dir.join("no_manifest")).unwrap();

    assert!(scan_pets(&pets_dir).is_empty());
  }

  #[test]
  fn test_scan_pets_skips_files() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();
    fs::write(pets_dir.join("not_a_pet"), "").unwrap();

    assert!(scan_pets(&pets_dir).is_empty());
  }

  #[test]
  fn test_scan_pets_finds_valid() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    fs::create_dir(pets_dir.join("zebra")).unwrap();
    fs::write(pets_dir.join("zebra").join("manifest.json"), "{}").unwrap();

    fs::create_dir(pets_dir.join("alpha")).unwrap();
    fs::write(pets_dir.join("alpha").join("manifest.json"), "{}").unwrap();

    let result = scan_pets(&pets_dir);
    assert_eq!(result, vec!["alpha", "zebra"]);
  }

  #[test]
  fn test_resolve_active_pet_empty_returns_empty_string() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    let result = resolve_active_pet(Some("ghost".into()), &pets_dir);
    assert_eq!(result, "");
  }

  #[test]
  fn test_resolve_active_pet_fallback_to_first() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    fs::create_dir(pets_dir.join("alpha")).unwrap();
    fs::write(pets_dir.join("alpha").join("manifest.json"), "{}").unwrap();

    // active pet does not exist, should fallback to first available
    let result = resolve_active_pet(Some("ghost".into()), &pets_dir);
    assert_eq!(result, "alpha");
  }

  #[test]
  fn test_resolve_active_pet_matching_exists() {
    let dir = TempDir::new().unwrap();
    let pets_dir = dir.path().join("pets");
    fs::create_dir(&pets_dir).unwrap();

    fs::create_dir(pets_dir.join("alpha")).unwrap();
    fs::write(pets_dir.join("alpha").join("manifest.json"), "{}").unwrap();

    fs::create_dir(pets_dir.join("beta")).unwrap();
    fs::write(pets_dir.join("beta").join("manifest.json"), "{}").unwrap();

    let result = resolve_active_pet(Some("beta".into()), &pets_dir);
    assert_eq!(result, "beta");
  }

  // ── AppSettings ──

  #[test]
  fn test_settings_default() {
    let s = AppSettings::default();
    assert_eq!(s.active_pet, Some("default-cat".into()));
    assert_eq!(s.always_on_top, Some(true));
    assert_eq!(s.scale, Some(5));
    assert_eq!(s.language, Some("zh".into()));
    assert_eq!(s.autostart, Some(false));
    assert_eq!(s.pets_dir, None);
  }

  #[test]
  fn test_settings_serde_roundtrip() {
    let original = AppSettings {
      pets_dir: Some("/custom/path".into()),
      active_pet: Some("MP".into()),
      always_on_top: Some(false),
      scale: Some(3),
      language: Some("en".into()),
      autostart: Some(true),
    };
    let json = serde_json::to_string_pretty(&original).unwrap();
    let restored: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.pets_dir, original.pets_dir);
    assert_eq!(restored.active_pet, original.active_pet);
    assert_eq!(restored.always_on_top, original.always_on_top);
    assert_eq!(restored.scale, original.scale);
    assert_eq!(restored.language, original.language);
    assert_eq!(restored.autostart, original.autostart);
  }

  #[test]
  fn test_settings_serde_partial() {
    // Simulates an older settings file missing some fields
    let json = r#"{"active_pet":"1234"}"#;
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert_eq!(s.active_pet, Some("1234".into()));
    assert_eq!(s.scale, None); // serde default for Option is None
  }
}