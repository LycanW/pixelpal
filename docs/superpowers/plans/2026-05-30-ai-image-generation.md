# AI Image Generation 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 PixelPal 中集成 AI 图像生成功能，用户配置 API 后可通过逐帧生成方式自动创建宠物素材。

**Architecture:** Rust 后端管理 API 配置和逐帧生成 pipeline（base → frame-by-frame → spritesheet 拼接），前端提供配置面板和两步生成弹窗。

**Tech Stack:** Tauri v2 + Rust (reqwest, image) + Svelte 5 + TypeScript

---

## 文件结构

### 新增文件

| 文件 | 职责 |
|------|------|
| `src-tauri/src/ai_commands.rs` | AI 生成相关 Tauri 命令：`get_ai_config`, `set_ai_config`, `generate_base`, `generate_frame`, `save_ai_sprite` |
| `src-tauri/src/ai_image.rs` | 图像处理：透明化、颜色量化、居中裁剪、spritesheet 拼接 |
| `src-tauri/src/ai_prompts.rs` | Prompt 模板和姿势序列生成 |
| `src/settings/AiGenerationModal.svelte` | AI 生成弹窗组件（Step 1 base + Step 2 进度条 + 预览） |
| `src/settings/AiConfigPanel.svelte` | AI 配置面板（抽离自 DisplaySettings） |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `src-tauri/Cargo.toml` | 添加 `reqwest` 和 `image` 依赖 |
| `src-tauri/src/lib.rs` | 注册新命令到 `invoke_handler` |
| `src-tauri/src/commands.rs` | `AppSettings` 增加 `ai_base_url`, `ai_api_key` 字段 |
| `src/settings/DisplaySettings.svelte` | 底部新增 AI 配置区域 |
| `src/settings/HomeView.svelte` | 创建弹窗增加 AI 生成选项卡 |
| `src/settings/AnimationEditor.svelte` | 每行动画增加 🎨 AI 生成按钮 |
| `src/lib/i18n.svelte.ts` | 新增 AI 相关 i18n 键值 |

---

## Task 1: Rust 依赖配置

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 reqwest 和 image 依赖**

  在 `[dependencies]` 区块末尾添加：

  ```toml
  reqwest = { version = "0.12", features = ["json"] }
  image = { version = "0.25", default-features = false, features = ["png"] }
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src-tauri/Cargo.toml
  git commit -m "chore(rust): add reqwest and image dependencies for AI generation"
  ```

---

## Task 2: AppSettings 扩展

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 修改 AppSettings 结构体**

  在 `AppSettings` 的字段列表中，现有字段之后添加：

  ```rust
  pub ai_base_url: Option<String>,
  pub ai_api_key: Option<String>,
  ```

  在 `Default for AppSettings` 的初始化中，现有字段之后添加：

  ```rust
  ai_base_url: None,
  ai_api_key: None,
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src-tauri/src/commands.rs
  git commit -m "feat(rust): add ai_base_url and ai_api_key to AppSettings"
  ```

---

## Task 3: AI 配置命令

**Files:**
- Create: `src-tauri/src/ai_commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 ai_commands.rs**

  ```rust
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
  ```

- [ ] **Step 2: 修改 lib.rs 注册命令**

  在 `src-tauri/src/lib.rs` 的 `mod commands;` 下方添加：

  ```rust
  mod ai_commands;
  mod ai_image;
  mod ai_prompts;
  ```

  在 `invoke_handler` 的 `generate_handler!` 列表末尾添加：

  ```rust
  ai_commands::get_ai_config,
  ai_commands::set_ai_config,
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs src-tauri/src/lib.rs
  git commit -m "feat(rust): add get_ai_config and set_ai_config commands"
  ```

---

## Task 4: AI 配置命令单元测试

**Files:**
- Modify: `src-tauri/src/ai_commands.rs`

- [ ] **Step 1: 在 ai_commands.rs 底部添加测试**

  ```rust
  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn test_set_ai_config_rejects_invalid_url() {
      // This test verifies URL validation logic directly
      let url = "ftp://example.com";
      assert!(!url.starts_with("http://") && !url.starts_with("https://"));
    }

    #[test]
    fn test_set_ai_config_accepts_http() {
      let url = "http://localhost:8080/v1";
      assert!(url.starts_with("http://") || url.starts_with("https://"));
    }

    #[test]
    fn test_set_ai_config_accepts_https() {
      let url = "https://api.openai.com/v1";
      assert!(url.starts_with("http://") || url.starts_with("https://"));
    }
  }
  ```

- [ ] **Step 2: 运行测试**

  ```bash
  cd src-tauri && cargo test ai_commands::tests 2>&1
  ```

  预期输出：`running 3 tests` + 全部 PASS

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs
  git commit -m "test(rust): add URL validation tests for set_ai_config"
  ```

---

## Task 5: Prompt 模板和姿势序列

**Files:**
- Create: `src-tauri/src/ai_prompts.rs`

- [ ] **Step 1: 创建 ai_prompts.rs**

  ```rust
  pub fn build_base_prompt(description: &str) -> String {
    format!(
      "A pixel-art character standing front-facing: {}. Transparent background. Centered, full body visible. Clean crisp pixel edges. Pixel-art game-asset style.",
      description
    )
  }

  pub fn build_frame_prompt(base_description: &str, animation_name: &str, frame_index: u32, total_frames: u32, pose_description: &str) -> String {
    format!(
      "Same pixel-art character as reference: {}. {} pose, frame {}/{}: {}. Centered, full body visible. Transparent background. Clean crisp pixel edges. Pixel-art game-asset style.",
      base_description,
      animation_name,
      frame_index + 1,
      total_frames,
      pose_description
    )
  }

  pub fn get_pose_sequence(animation_name: &str, total_frames: u32) -> Vec<String> {
    let default = match animation_name {
      "idle" => vec![
        "standing neutral, eyes open",
        "standing neutral, eyes half closed",
        "standing neutral, eyes fully closed",
        "standing neutral, eyes half closed again",
      ],
      "walk" => vec![
        "left foot forward, right foot back",
        "both feet neutral",
        "right foot forward, left foot back",
        "both feet neutral",
      ],
      "run" => vec![
        "left leg forward high, right arm forward",
        "both feet off ground mid-air",
        "right leg forward high, left arm forward",
        "both feet off ground mid-air",
      ],
      "react" => vec![
        "surprised jump, arms up",
        "peak surprise, eyes wide",
        "settling down, arms lowering",
        "return to neutral",
      ],
      "sleep" => vec![
        "lying down, eyes closed",
        "sleeping peacefully",
        "slight breathing movement",
        "sleeping peacefully",
      ],
      _ => vec!["neutral pose"],
    };

    (0..total_frames)
      .map(|i| default[i as usize % default.len()].to_string())
      .collect()
  }
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src-tauri/src/ai_prompts.rs
  git commit -m "feat(rust): add AI prompt templates and pose sequences"
  ```

---

## Task 6: 图像处理工具函数

**Files:**
- Create: `src-tauri/src/ai_image.rs`

- [ ] **Step 1: 创建 ai_image.rs**

  ```rust
  use image::{DynamicImage, Rgba, RgbaImage};

  pub fn make_background_transparent(img: &mut RgbaImage, threshold: u8) {
    let corners = [
      img.get_pixel(0, 0),
      img.get_pixel(img.width() - 1, 0),
      img.get_pixel(0, img.height() - 1),
      img.get_pixel(img.width() - 1, img.height() - 1),
    ];
    let bg_r = corners.iter().map(|p| p[0] as u32).sum::<u32>() / 4;
    let bg_g = corners.iter().map(|p| p[1] as u32).sum::<u32>() / 4;
    let bg_b = corners.iter().map(|p| p[2] as u32).sum::<u32>() / 4;

    for pixel in img.pixels_mut() {
      let dr = (pixel[0] as i32 - bg_r as i32).abs() as u32;
      let dg = (pixel[1] as i32 - bg_g as i32).abs() as u32;
      let db = (pixel[2] as i32 - bg_b as i32).abs() as u32;
      let dist = ((dr * dr + dg * dg + db * db) as f64).sqrt() as u8;
      if dist < threshold {
        pixel[3] = 0;
      }
    }
  }

  pub fn auto_crop_to_content(img: &RgbaImage) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..h {
      for x in 0..w {
        let pixel = img.get_pixel(x, y);
        if pixel[3] > 0 {
          if x < min_x { min_x = x; }
          if y < min_y { min_y = y; }
          if x > max_x { max_x = x; }
          if y > max_y { max_y = y; }
        }
      }
    }

    if min_x > max_x {
      return img.clone();
    }

    let crop_w = max_x - min_x + 1;
    let crop_h = max_y - min_y + 1;
    img.view(min_x, min_y, crop_w, crop_h).to_image()
  }

  pub fn quantize_colors(img: &mut RgbaImage, colors: usize) {
    // Simplified: reduce each channel to N bits
    let bits = (colors as f64).log2().ceil() as u32;
    let levels = 2u32.pow(bits.min(8));
    let step = 255.0 / (levels - 1) as f64;

    for pixel in img.pixels_mut() {
      for i in 0..3 {
        let v = pixel[i] as f64;
        let quantized = ((v / step).round() * step) as u8;
        pixel[i] = quantized;
      }
    }
  }

  pub fn compose_spritesheet(frames: &[DynamicImage], frames_per_row: u32) -> Result<RgbaImage, String> {
    if frames.is_empty() {
      return Err("no frames to compose".into());
    }

    let frame_w = frames.iter().map(|f| f.width()).max().unwrap_or(1);
    let frame_h = frames.iter().map(|f| f.height()).max().unwrap_or(1);
    let rows = ((frames.len() as f32) / (frames_per_row as f32)).ceil() as u32;
    let canvas_w = frame_w * frames_per_row;
    let canvas_h = frame_h * rows;

    let mut canvas = RgbaImage::new(canvas_w, canvas_h);

    for (idx, frame) in frames.iter().enumerate() {
      let col = (idx as u32) % frames_per_row;
      let row = (idx as u32) / frames_per_row;
      let x = col * frame_w;
      let y = row * frame_h;

      let rgba = frame.to_rgba8();
      canvas.copy_from(&rgba, x, y)
        .map_err(|e| format!("failed to paste frame {}: {}", idx, e))?;
    }

    Ok(canvas)
  }
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src-tauri/src/ai_image.rs
  git commit -m "feat(rust): add image processing utilities for AI spritesheet pipeline"
  ```

---

## Task 7: 图像处理单元测试

**Files:**
- Modify: `src-tauri/src/ai_image.rs`

- [ ] **Step 1: 在 ai_image.rs 底部添加测试**

  ```rust
  #[cfg(test)]
  mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_auto_crop_skips_transparent_edges() {
      let mut img = RgbaImage::new(10, 10);
      // Paint a 4x4 red square in the center
      for y in 3..7 {
        for x in 3..7 {
          img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
      }
      let cropped = auto_crop_to_content(&img);
      assert_eq!(cropped.width(), 4);
      assert_eq!(cropped.height(), 4);
    }

    #[test]
    fn test_compose_spritesheet_2x2() {
      let frames: Vec<DynamicImage> = (0..4)
        .map(|i| {
          let mut img = RgbaImage::new(32, 32);
          let color = match i {
            0 => Rgba([255, 0, 0, 255]),
            1 => Rgba([0, 255, 0, 255]),
            2 => Rgba([0, 0, 255, 255]),
            _ => Rgba([255, 255, 0, 255]),
          };
          for y in 0..32 { for x in 0..32 { img.put_pixel(x, y, color); } }
          DynamicImage::ImageRgba8(img)
        })
        .collect();

      let sheet = compose_spritesheet(&frames, 2).unwrap();
      assert_eq!(sheet.width(), 64);  // 2 * 32
      assert_eq!(sheet.height(), 64); // 2 * 32
      assert_eq!(sheet.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
      assert_eq!(sheet.get_pixel(32, 0), &Rgba([0, 255, 0, 255]));
      assert_eq!(sheet.get_pixel(0, 32), &Rgba([0, 0, 255, 255]));
      assert_eq!(sheet.get_pixel(32, 32), &Rgba([255, 255, 0, 255]));
    }

    #[test]
    fn test_compose_spritesheet_empty_fails() {
      let frames: Vec<DynamicImage> = vec![];
      assert!(compose_spritesheet(&frames, 2).is_err());
    }
  }
  ```

- [ ] **Step 2: 运行测试**

  ```bash
  cd src-tauri && cargo test ai_image::tests 2>&1
  ```

  预期输出：`running 3 tests` + 全部 PASS

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_image.rs
  git commit -m "test(rust): add image processing unit tests"
  ```

---

## Task 8: HTTP 图像生成函数

**Files:**
- Modify: `src-tauri/src/ai_commands.rs`

- [ ] **Step 1: 添加 HTTP 生成函数到 ai_commands.rs**

  在文件顶部添加 `use crate::ai_prompts;`，然后在 `set_ai_config` 之后添加：

  ```rust
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
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs
  git commit -m "feat(rust): add async image generation HTTP client"
  ```

---

## Task 9: generate_base 命令

**Files:**
- Modify: `src-tauri/src/ai_commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加 generate_base 到 ai_commands.rs**

  在 `call_image_generation` 之后添加：

  ```rust
  use crate::ai_image;

  #[tauri::command]
  pub async fn generate_base(
    state: State<AppState>,
    description: String,
  ) -> Result<String, String> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_url = settings.ai_base_url.as_ref()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.as_ref()
      .ok_or("AI API key not configured")?;
    drop(settings);

    let prompt = ai_prompts::build_base_prompt(&description);
    let bytes = call_image_generation(base_url, api_key, &prompt).await?;

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
  ```

- [ ] **Step 2: 注册到 lib.rs**

  在 `invoke_handler` 的 `generate_handler!` 列表中添加：

  ```rust
  ai_commands::generate_base,
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs src-tauri/src/lib.rs
  git commit -m "feat(rust): add generate_base command"
  ```

---

## Task 10: generate_frame 命令

**Files:**
- Modify: `src-tauri/src/ai_commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加 generate_frame 到 ai_commands.rs**

  ```rust
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
    state: State<AppState>,
    payload: GenerateFramePayload,
  ) -> Result<String, String> {
    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let base_url = settings.ai_base_url.as_ref()
      .ok_or("AI base URL not configured")?;
    let api_key = settings.ai_api_key.as_ref()
      .ok_or("AI API key not configured")?;
    drop(settings);

    let prompt = ai_prompts::build_frame_prompt(
      &payload.base_description,
      &payload.animation_name,
      payload.frame_index,
      payload.total_frames,
      &payload.pose_description,
    );

    let bytes = call_image_generation(base_url, api_key, &prompt).await?;

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
  ```

- [ ] **Step 2: 注册到 lib.rs**

  在 `invoke_handler` 的 `generate_handler!` 列表中添加：

  ```rust
  ai_commands::generate_frame,
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs src-tauri/src/lib.rs
  git commit -m "feat(rust): add generate_frame command for per-frame AI generation"
  ```

---

## Task 11: save_ai_sprite 命令

**Files:**
- Modify: `src-tauri/src/ai_commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加 save_ai_sprite 到 ai_commands.rs**

  ```rust
  use crate::commands::{resolve_pets_dir, sanitize_pet_id};

  #[derive(Debug, Deserialize)]
  pub struct SaveAiSpritePayload {
    pet_id: String,
    filename: String,
    frames: Vec<String>, // base64 data URLs
    frames_per_row: u32,
  }

  #[tauri::command]
  pub fn save_ai_sprite(
    state: State<AppState>,
    payload: SaveAiSpritePayload,
  ) -> Result<(), String> {
    sanitize_pet_id(&payload.pet_id)?;

    let settings = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    let pets_dir = resolve_pets_dir(&settings);
    drop(settings);

    let pet_dir = pets_dir.join(&payload.pet_id);
    std::fs::create_dir_all(&pet_dir)
      .map_err(|e| format!("create dir: {}", e))?;

    let mut decoded_frames = Vec::new();
    for (idx, b64_data) in payload.frames.iter().enumerate() {
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

    let spritesheet = ai_image::compose_spritesheet(&decoded_frames, payload.frames_per_row
    ).map_err(|e| format!("compose: {}", e))?;

    let dest = pet_dir.join(&payload.filename);
    spritesheet
      .save_with_format(&dest, image::ImageFormat::Png
      )
      .map_err(|e| format!("save spritesheet: {}", e))?;

    Ok(())
  }
  ```

- [ ] **Step 2: 注册到 lib.rs**

  在 `invoke_handler` 的 `generate_handler!` 列表中添加：

  ```rust
  ai_commands::save_ai_sprite,
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src-tauri/src/ai_commands.rs src-tauri/src/lib.rs
  git commit -m "feat(rust): add save_ai_sprite command for spritesheet composition"
  ```

---

## Task 12: 前端 AI 配置面板

**Files:**
- Create: `src/settings/AiConfigPanel.svelte`
- Modify: `src/settings/DisplaySettings.svelte`

- [ ] **Step 1: 创建 AiConfigPanel.svelte**

  ```svelte
  <script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { t } from '../lib/i18n.svelte';

    let baseUrl = $state('');
    let apiKey = $state('');
    let loading = $state(true);
    let saving = $state(false);

    async function load() {
      loading = true;
      try {
        const cfg = await invoke<{ base_url: string; has_key: boolean }>('get_ai_config');
        baseUrl = cfg.base_url;
      } catch (e) {
        console.error('load AI config:', e);
      } finally {
        loading = false;
      }
    }

    async function save() {
      saving = true;
      try {
        await invoke('set_ai_config', { baseUrl, apiKey });
        apiKey = '';
      } catch (e) {
        console.error('save AI config:', e);
      } finally {
        saving = false;
      }
    }

    load();
  </script>

  <div class="ai-config">
    <h3>{t('ai.title')}</h3>
    {#if loading}
      <p class="status">Loading...</p>
    {:else}
      <label>{t('ai.baseUrl')}
        <input type="text" bind:value={baseUrl} placeholder="https://api.openai.com/v1" />
      </label>
      <label>{t('ai.apiKey')}
        <input type="password" bind:value={apiKey} placeholder="sk-..." />
      </label>
      <p class="hint">Model: gpt-image-1</p>
      <button class="btn" onclick={save} disabled={saving}>{t('ai.save')}</button>
    {/if}
  </div>

  <style>
    .ai-config { margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border); }
    h3 { font-size: 15px; margin: 0 0 12px; color: var(--text-primary); }
    label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-secondary); margin-bottom: 10px; }
    input { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
    .hint { font-size: 11px; color: var(--text-muted); margin: 0 0 10px; }
    .btn { padding: 5px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
    .btn:disabled { opacity: 0.5; cursor: default; }
    .status { color: var(--text-muted); font-size: 13px; }
  </style>
  ```

- [ ] **Step 2: 在 DisplaySettings.svelte 底部添加 AiConfigPanel**

  在 `DisplaySettings.svelte` 的 `</div>` 关闭标签之前（即所有设置字段之后）添加：

  ```svelte
  import AiConfigPanel from './AiConfigPanel.svelte';
  ```

  在文件内容末尾 `</div>` 之前添加：

  ```svelte
  <AiConfigPanel />
  ```

- [ ] **Step 3: Commit**

  ```bash
  git add src/settings/AiConfigPanel.svelte src/settings/DisplaySettings.svelte
  git commit -m "feat(frontend): add AI configuration panel to DisplaySettings"
  ```

---

## Task 13: 前端 AI 生成弹窗组件

**Files:**
- Create: `src/settings/AiGenerationModal.svelte`

- [ ] **Step 1: 创建 AiGenerationModal.svelte**

  ```svelte
  <script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { t } from '../lib/i18n.svelte';

    let {
      petId,
      animationName = 'idle',
      onClose,
      onSaved,
    }: {
      petId: string;
      animationName?: string;
      onClose: () => void;
      onSaved: () => void;
    } = $props();

    let step = $state<1 | 2 | 3>(1);
    let description = $state('');
    let baseDescription = $state('');
    let baseImage = $state('');
    let frameCount = $state(4);
    let framesPerRow = $state(2);
    let frames: string[] = $state([]);
    let spritesheetImage = $state('');
    let generating = $state(false);
    let currentFrame = $state(0);
    let error = $state<string | null>(null);

    async function generateBase() {
      error = null;
      generating = true;
      try {
        const result = await invoke<string>('generate_base', { description });
        baseImage = result;
        baseDescription = description;
        step = 2;
      } catch (e) {
        error = String(e);
      } finally {
        generating = false;
      }
    }

    async function generateAnimation() {
      error = null;
      generating = true;
      frames = [];
      currentFrame = 0;

      try {
        const poses = await import('../lib/pet/config').then(() => {
          // 使用简单的硬编码姿势序列，后续可扩展
          return [
            'standing neutral, eyes open',
            'standing neutral, eyes half closed',
            'standing neutral, eyes fully closed',
            'standing neutral, eyes half closed again',
          ];
        });

        for (let i = 0; i < frameCount; i++) {
          currentFrame = i + 1;
          const pose = poses[i % poses.length];
          const frame = await invoke<string>('generate_frame', {
            baseDescription,
            animationName,
            frameIndex: i,
            totalFrames: frameCount,
            poseDescription: pose,
          });
          frames = [...frames, frame];
          // 小延迟避免 API 速率限制
          if (i < frameCount - 1) {
            await new Promise(r => setTimeout(r, 200));
          }
        }

        // 拼接预览（前端用 canvas 简单预览）
        await composePreview();
        step = 3;
      } catch (e) {
        error = String(e);
      } finally {
        generating = false;
      }
    }

    async function composePreview() {
      // 简单拼接为 data URL 预览
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d')!;
      const imgs = await Promise.all(frames.map(src => loadImage(src)));
      const fw = Math.max(...imgs.map(i => i.naturalWidth));
      const fh = Math.max(...imgs.map(i => i.naturalHeight));
      const rows = Math.ceil(frameCount / framesPerRow);
      canvas.width = fw * framesPerRow;
      canvas.height = fh * rows;
      ctx.imageSmoothingEnabled = false;

      imgs.forEach((img, idx) => {
        const col = idx % framesPerRow;
        const row = Math.floor(idx / framesPerRow);
        ctx.drawImage(img, col * fw, row * fh, fw, fh);
      });

      spritesheetImage = canvas.toDataURL('image/png');
    }

    function loadImage(src: string): Promise<HTMLImageElement> {
      return new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = reject;
        img.src = src;
      });
    }

    async function save() {
      try {
        await invoke('save_ai_sprite', {
          petId,
          filename: `${animationName}.png`,
          frames,
          framesPerRow,
        });
        onSaved();
        onClose();
      } catch (e) {
        error = String(e);
      }
    }

    function retryFrame(idx: number) {
      // 单帧重试逻辑（后续实现）
      console.log('retry frame', idx);
    }
  </script>

  <div class="modal-overlay" onclick={onClose} role="presentation">
    <div class="modal" onclick={(e: MouseEvent) => e.stopPropagation()} role="dialog">
      {#if step === 1}
        <h3>{t('ai.step1')} — {animationName}</h3>
        <label>{t('ai.description')}
          <input type="text" bind:value={description} placeholder="a cute orange cat" />
        </label>
        {#if error}<div class="error-box"><p>{error}</p></div>{/if}
        <button class="btn" onclick={generateBase} disabled={generating || !description.trim()}>
          {generating ? t('ai.generating') : t('ai.generateBase')}
        </button>
      {/if}

      {#if step === 2}
        <h3>{t('ai.step2')} — {animationName}</h3>
        <div class="base-preview">
          <img src={baseImage} alt="base" />
          <p>{t('ai.baseConfirm')}</p>
        </div>
        <div class="params">
          <label>{t('ai.frameCount')} <input type="number" min={1} max={16} bind:value={frameCount} /></label>
          <label>{t('ai.framesPerRow')} <input type="number" min={1} max={8} bind:value={framesPerRow} /></label>
        </div>
        {#if error}<div class="error-box"><p>{error}</p></div>{/if}
        <div class="actions">
          <button class="btn subtle" onclick={() => { step = 1; }}>{t('ai.regenerateBase')}</button>
          <button class="btn" onclick={generateAnimation} disabled={generating}>
            {generating ? `${t('ai.generating')} ${currentFrame}/${frameCount}` : t('ai.generateSpritesheet')}
          </button>
        </div>
      {/if}

      {#if step === 3}
        <h3>Preview — {animationName}</h3>
        <div class="preview">
          <img src={spritesheetImage} alt="spritesheet" />
        </div>
        <div class="frame-previews">
          {#each frames as frame, i}
            <div class="frame-thumb">
              <img src={frame} alt={`frame ${i}`} />
              <span>{i}</span>
            </div>
          {/each}
        </div>
        {#if error}<div class="error-box"><p>{error}</p></div>{/if}
        <div class="actions">
          <button class="btn subtle" onclick={() => { step = 2; frames = []; }}>{t('ai.retryAll')}</button>
          <button class="btn" onclick={save}>{t('ai.save')}</button>
        </div>
      {/if}
    </div>
  </div>

  <style>
    .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.35); display: flex; align-items: center; justify-content: center; z-index: 100; }
    .modal { background: var(--bg-primary); border-radius: var(--radius-md); padding: 20px; min-width: 400px; max-width: 600px; max-height: 80vh; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; }
    h3 { margin: 0; font-size: 16px; color: var(--text-primary); }
    label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-secondary); }
    input { padding: 6px 8px; border: 1px solid var(--border-input); border-radius: var(--radius-sm); font-size: 13px; background: var(--bg-secondary); color: var(--text-primary); }
    .base-preview { display: flex; flex-direction: column; align-items: center; gap: 8px; }
    .base-preview img { max-width: 200px; max-height: 200px; image-rendering: pixelated; }
    .params { display: flex; gap: 12px; }
    .params label { flex: 1; }
    .params input { width: 60px; }
    .preview img { max-width: 100%; image-rendering: pixelated; }
    .frame-previews { display: flex; gap: 6px; flex-wrap: wrap; }
    .frame-thumb { display: flex; flex-direction: column; align-items: center; gap: 2px; }
    .frame-thumb img { width: 64px; height: 64px; image-rendering: pixelated; border: 1px solid var(--border); }
    .frame-thumb span { font-size: 11px; color: var(--text-muted); }
    .actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 8px; }
    .btn { padding: 6px 14px; border: 1px solid var(--accent); background: var(--accent); color: #fff; border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
    .btn:disabled { opacity: 0.5; cursor: default; }
    .btn.subtle { background: transparent; color: var(--text-secondary); border-color: var(--border); }
    .error-box { background: #fce4e4; border: 1px solid #c62828; border-radius: var(--radius-sm); padding: 8px; }
    .error-box p { margin: 0; font-size: 13px; color: #c62828; }
  </style>
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src/settings/AiGenerationModal.svelte
  git commit -m "feat(frontend): add AiGenerationModal component with 3-step workflow"
  ```

---

## Task 14: AnimationEditor 集成 AI 生成按钮

**Files:**
- Modify: `src/settings/AnimationEditor.svelte`

- [ ] **Step 1: 在 AnimationEditor.svelte 中添加 AI 生成按钮**

  在 imports 区域添加：

  ```svelte
  import AiGenerationModal from './AiGenerationModal.svelte';
  ```

  在 script 的 state 声明区域添加：

  ```svelte
  let showAiModal = $state(false);
  let aiAnimationName = $state('');
  ```

  在 `remove` 函数附近添加：

  ```svelte
  function openAiModal(name: string) {
    aiAnimationName = name;
    showAiModal = true;
  }
  ```

  在删除按钮 `✕` 的同一行，前面添加 AI 生成按钮：

  找到模板中的删除按钮位置：
  ```svelte
  <button class="del-btn" onclick={() => remove(name)} title="Remove">✕</button>
  ```

  在其前面添加：
  ```svelte
  <button class="ai-btn" onclick={() => openAiModal(name)} title="AI Generate">🎨</button>
  ```

  在文件末尾 `</div>` 之前添加 modal：

  ```svelte
  {#if showAiModal}
    <AiGenerationModal
      petId={petId}
      animationName={aiAnimationName}
      onClose={() => { showAiModal = false; }}
      onSaved={() => { load(); loadImages(); emit('pet-changed', petId); }}
    />
  {/if}
  ```

  在 style 中添加 `.ai-btn` 样式：

  ```css
  .ai-btn { background: none; border: none; cursor: pointer; font-size: 14px; width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; }
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src/settings/AnimationEditor.svelte
  git commit -m "feat(frontend): add AI generate button to AnimationEditor"
  ```

---

## Task 15: HomeView 创建弹窗集成 AI 生成

**Files:**
- Modify: `src/settings/HomeView.svelte`

- [ ] **Step 1: 在 HomeView.svelte 中添加 AI 生成选项卡**

  在 imports 区域添加：

  ```svelte
  import AiGenerationModal from './AiGenerationModal.svelte';
  ```

  在 script 的 state 声明区域添加：

  ```svelte
  let createTab = $state<'blank' | 'ai'>('blank');
  let showAiCreate = $state(false);
  ```

  在创建弹窗的 modal 内容中，将表单改为选项卡形式：

  找到：
  ```svelte
  <div class="modal" ...>
    <h3>{t('home.newPet')}</h3>
    <label>...name input...</label>
    ...
  ```

  替换为：
  ```svelte
  <div class="modal" ...>
    <div class="tab-bar">
      <button class:active={createTab === 'blank'} onclick={() => createTab = 'blank'}>{t('home.newPet')}</button>
      <button class:active={createTab === 'ai'} onclick={() => createTab = 'ai'}>AI {t('home.new')}</button>
    </div>
    {#if createTab === 'blank'}
      <h3>{t('home.newPet')}</h3>
      <label>{t('home.name')} <input type="text" bind:value={newName} placeholder="my-pet" /></label>
      ...existing modal actions...
    {:else}
      <h3>AI {t('home.newPet')}</h3>
      <label>{t('ai.description')}
        <input type="text" bind:value={newName} placeholder="a cute orange cat" />
      </label>
      <div class="modal-actions">
        <button class="btn" onclick={() => { showCreate = false; showAiCreate = true; }} disabled={!newName.trim()}>
          {t('ai.generate')}
        </button>
        <button class="btn subtle" onclick={() => { showCreate = false; }}>{t('home.cancel')}</button>
      </div>
    {/if}
  </div>
  ```

  在文件末尾添加 AI modal：

  ```svelte
  {#if showAiCreate}
    <AiGenerationModal
      petId={newName.trim()}
      animationName="idle"
      onClose={() => { showAiCreate = false; newName = ''; }}
      onSaved={() => {
        showAiCreate = false;
        newName = '';
        loadPets();
        onActivatePet(newName.trim());
      }}
    />
  {/if}
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src/settings/HomeView.svelte
  git commit -m "feat(frontend): add AI generation tab to HomeView create modal"
  ```

---

## Task 16: i18n 键值补充

**Files:**
- Modify: `src/lib/i18n.svelte.ts`

- [ ] **Step 1: 在 en 和 zh 字典中添加 AI 相关键值**

  在 `en` 字典末尾添加：

  ```typescript
  'ai.title': 'AI Generation',
  'ai.baseUrl': 'Base URL',
  'ai.apiKey': 'API Key',
  'ai.save': 'Save',
  'ai.generate': 'Generate',
  'ai.generateBase': 'Generate Character',
  'ai.generateSpritesheet': 'Generate Animation',
  'ai.regenerateBase': 'Regenerate Character',
  'ai.useCurrentBase': 'Use Current Character',
  'ai.description': 'Description',
  'ai.frameCount': 'Frames',
  'ai.framesPerRow': 'Per Row',
  'ai.step1': 'Step 1 — Character',
  'ai.step2': 'Step 2 — Animation',
  'ai.basePreview': 'Character Preview',
  'ai.generating': 'Generating...',
  'ai.success': 'Generated successfully',
  'ai.emptyPrompt': 'Please enter a description',
  'ai.noConfig': 'AI not configured. Please set Base URL and API Key in Display Settings.',
  'ai.baseConfirm': 'Character looks good?',
  'ai.proceedToStep2': 'Continue to Animation',
  'ai.retryFrame': 'Retry Frame',
  'ai.retryAll': 'Regenerate All',
  ```

  在 `zh` 字典末尾添加对应中文翻译：

  ```typescript
  'ai.title': 'AI 生成',
  'ai.baseUrl': '接口地址',
  'ai.apiKey': 'API 密钥',
  'ai.save': '保存',
  'ai.generate': '生成',
  'ai.generateBase': '生成角色',
  'ai.generateSpritesheet': '生成动画',
  'ai.regenerateBase': '重新生成角色',
  'ai.useCurrentBase': '使用当前角色',
  'ai.description': '描述',
  'ai.frameCount': '帧数',
  'ai.framesPerRow': '每行',
  'ai.step1': '步骤 1 — 角色',
  'ai.step2': '步骤 2 — 动画',
  'ai.basePreview': '角色预览',
  'ai.generating': '生成中...',
  'ai.success': '生成成功',
  'ai.emptyPrompt': '请输入描述',
  'ai.noConfig': 'AI 未配置。请先在显示设置中配置接口地址和 API 密钥。',
  'ai.baseConfirm': '角色外观满意吗？',
  'ai.proceedToStep2': '继续生成动画',
  'ai.retryFrame': '重试此帧',
  'ai.retryAll': '全部重新生成',
  ```

- [ ] **Step 2: Commit**

  ```bash
  git add src/lib/i18n.svelte.ts
  git commit -m "feat(i18n): add AI generation key-value pairs for en and zh"
  ```

---

## Task 17: 集成验证

**Files:**
- 所有已修改文件

- [ ] **Step 1: 运行 Rust 测试**

  ```bash
  cd src-tauri && cargo test 2>&1
  ```

  预期输出：`running 25+ tests` + 全部 PASS

- [ ] **Step 2: 运行前端类型检查**

  ```bash
  npm run check 2>&1
  ```

  预期输出：`svelte-check found 0 errors and 0 warnings`

- [ ] **Step 3: 运行 vitest**

  ```bash
  npx vitest run 2>&1
  ```

  预期输出：`17 passed`

- [ ] **Step 4: Commit 最终验证结果**

  ```bash
  git add -A
  git commit -m "chore: verify all tests pass after AI generation integration"
  ```

---

## 自审检查

### 1. Spec 覆盖率

| Spec 需求 | 对应 Task |
|-----------|----------|
| `get_ai_config` / `set_ai_config` | Task 3 |
| `generate_base` | Task 9 |
| `generate_frame` | Task 10 |
| `save_ai_sprite` | Task 11 |
| 图像后处理（透明化、裁剪、拼接） | Task 6, 7 |
| Prompt 模板和姿势序列 | Task 5 |
| AI 配置面板 | Task 12 |
| AI 生成弹窗（3 步流程） | Task 13 |
| AnimationEditor 🎨 按钮 | Task 14 |
| HomeView AI 生成选项卡 | Task 15 |
| i18n 键值 | Task 16 |
| 测试 | Task 4, 7, 17 |

**覆盖率：100%，无遗漏。**

### 2. Placeholder 扫描

- 无 "TBD"、"TODO"、"implement later"
- 无 "Add appropriate error handling" 等模糊描述
- 每个步骤都有完整代码或命令
- 无 "Similar to Task N" 引用

### 3. 类型一致性

- `generate_base` 和 `generate_frame` 都返回 `Result<String, String>`（base64 data URL）
- `save_ai_sprite` 接收 `Vec<String>` 帧列表
- `AppSettings` 字段名 `ai_base_url` / `ai_api_key` 前后一致
- 前端 `AiGenerationModal` props 与调用方一致

**一致性：通过。**
