# PixelPal AI 图像生成设计文档

## 目标

在 PixelPal 中集成 AI 图像生成功能，降低宠物创作门槛。用户自行提供 OpenAI 兼容的 API base URL 和 API key，系统根据描述自动生成像素风角色素材，自动适配现有动画系统。

## 核心设计原则

- **渐进增强**：AI 生成是现有工作流的补充，不替代手动创作
- **两种模式**：创建时一键生成（快速体验）+ 编辑时单动画生成（精细控制）
- **最小侵入**：复用现有 `AnimationController`、`SpriteRenderer` 和 `config.json` 格式
- **安全优先**：API key 只存 Rust 后端，前端不持久化密钥
- **精确约束**：学 hatch-pet 逐帧分别生成 + 后端精确拼接，避免网格对齐问题

## 架构概述

```
用户描述角色
    │
    ├─→ Step 1: 生成 Base 角色
    │    [generate_base(description)]
    │    │
    │    ▼
    │  HTTP POST /v1/images/generations
    │    │
    │    ▼
    │  返回 1024×1024 base 图 PNG
    │    │
    │    ▼
    │  用户预览 base 图 → 满意？继续 / 不满意？重试
    │    │
    │    ▼
    │  Rust 提取角色特征描述（颜色、外形）
    │    │
    ├─→ Step 2: 逐帧生成动画帧
    │    [generate_frame(base_desc, anim_name, frame_idx, total, pose_desc)]
    │    │
    │    ▼
    │  串行或并行 HTTP POST × frame_count 次
    │    │
    │    ▼
    │  返回 frame_count 张单帧 PNG
    │    │
    │    ▼
    │  Rust 后处理：透明化、颜色量化、统一尺寸
    │    │
    │    ▼
    ▼
Rust 后端自动拼接成 spritesheet
    │
    ▼
前端预览弹窗（逐帧预览 + 整体 spritesheet）
    │
    ▼
用户确认 → [save_ai_sprite(frames, frames_per_row)] → 保存 PNG
    │
    ▼
自动创建/更新动画定义（frameCount, framesPerRow, source）
```

## Rust 后端

### 配置模型

`AppSettings` 新增两个字段：

```rust
pub struct AppSettings {
  // ... existing fields ...
  pub ai_base_url: Option<String>,   // e.g. "https://api.openai.com/v1"
  pub ai_api_key: Option<String>,    // stored in plain text in user data dir
}
```

- 默认值均为 `None`
- `ai_base_url` 为空时，生成按钮不可用
- `ai_api_key` 为空时，生成按钮不可用

### 新增命令

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get_ai_config` | — | `{ base_url: String, has_key: bool }` | 不返回 key 本身 |
| `set_ai_config` | `{ base_url: String, api_key: String }` | `Result<(), String>` | 验证 URL 格式 |
| `generate_base` | `{ description: String }` | `Result<String, String>` | Step 1：生成角色 base 图（1024×1024），返回 base64 PNG |
| `generate_frame` | `{ base_description: String, animation_name: String, frame_index: u32, total_frames: u32, pose_description: String }` | `Result<String, String>` | Step 2：生成单帧，返回后处理后的 base64 PNG |
| `save_ai_sprite` | `{ pet_id: String, filename: String, frames: Vec<String>, frames_per_row: u32 }` | `Result<(), String>` | 将多帧拼接成 spritesheet 并保存到宠物目录 |

#### Step 1: `generate_base` 实现细节

1. **构建请求体**（OpenAI images/generations 格式）：
   ```json
   {
     "model": "gpt-image-1",
     "prompt": "A pixel-art character standing front-facing: {description}. Transparent background. Centered, full body visible. Clean crisp pixel edges. Game-asset style.",
     "size": "1024x1024",
     "quality": "high",
     "n": 1
   }
   ```

2. **发送 HTTP 请求**，超时 60 秒
3. **解析响应**：提取 `data[0].b64_json`
4. **后处理**：透明化（见下方"后端图像后处理"章节）
5. **返回 base64 data URL**（保持 1024×1024 分辨率）

**base 图的作用**：
- 让用户先看到角色长什么样，满意再继续
- 提取角色特征描述（颜色、外形），作为逐帧生成的锚点
- 防止"生成完发现角色完全不对"的浪费

#### Step 2: `generate_frame` 实现细节

1. **确定姿势描述**：根据 `animation_name` 和 `frame_index` 从预定义姿势表中选取（见下方"姿势序列"章节），用户可覆盖
2. **构建请求体**：
   ```json
   {
     "model": "gpt-image-1",
     "prompt": "Same pixel-art character as reference: {base_description}. {animation_name} pose, frame {frame_index}/{total_frames}: {pose_description}. Centered, full body visible. Transparent background. Clean crisp pixel edges. Game-asset style.",
     "size": "1024x1024",
     "quality": "high",
     "n": 1
   }
   ```

3. **发送 HTTP 请求**，超时 60 秒
4. **解析响应**：提取 `data[0].b64_json`
5. **图像后处理**（见下方"后端图像后处理"章节）
6. **返回 base64 data URL**（保持 1024×1024 分辨率）

**调用策略**：
- 默认串行调用（避免 API 速率限制）
- 每帧之间有 200ms 间隔
- 前端显示进度：`Generating frame 3/4...`
- 任一帧失败则整体失败，返回具体错误

#### `save_ai_sprite` 实现细节

接收 `frames: Vec<String>`（每帧一个 base64 data URL）和 `frames_per_row`。

拼接流程：
1. 解码所有帧为 `image::RgbaImage`
2. 找到最大宽度和最大高度
3. 将所有帧统一缩放到相同尺寸（`max_width × max_height`）
4. 按 `frames_per_row` 计算行数：`rows = ceil(frames.len() / frames_per_row)`
5. 创建空白画布：`max_width × frames_per_row` 宽 × `max_height × rows` 高
6. 逐帧粘贴到对应网格位置
7. 保存为 PNG 到宠物目录

```rust
fn compose_spritesheet(
    frames: Vec<image::RgbaImage>,
    frames_per_row: u32,
) -> image::RgbaImage {
    let frame_w = frames.iter().map(|f| f.width()).max().unwrap_or(1);
    let frame_h = frames.iter().map(|f| f.height()).max().unwrap_or(1);
    let rows = ((frames.len() as f32) / (frames_per_row as f32)).ceil() as u32;
    let canvas_w = frame_w * frames_per_row;
    let canvas_h = frame_h * rows;
    
    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);
    for (idx, frame) in frames.iter().enumerate() {
        let col = (idx as u32) % frames_per_row;
        let row = (idx as u32) / frames_per_row;
        let x = col * frame_w;
        let y = row * frame_h;
        canvas.copy_from(frame, x, y).unwrap();
    }
    canvas
}
```

### 依赖变更

`Cargo.toml` 新增：
```toml
reqwest = { version = "0.12", features = ["json"] }
image = { version = "0.25", default-features = false, features = ["png"] }
```

## 前端

### 1. AI 配置面板

在 `DisplaySettings.svelte` 下方新增 "AI Generation" 区域：

```
[AI Generation]
Base URL: [https://api.openai.com/v1    ]
API Key:  [••••••••••••••••             ]
Model:    gpt-image-1 (固定，提示文字说明)
[Save]
```

- 两个输入框均可编辑
- API key 输入类型为 `password`（`type="password"`）
- 保存时调用 `set_ai_config`
- 加载时调用 `get_ai_config`（只显示 base_url，key 是否已设置用 has_key 判断）

### 2. HomeView — 创建宠物时的 AI 生成

`HomeView.svelte` 的创建弹窗增加选项卡：

```
[空白宠物] [AI 生成]

Step 1 — 生成角色：
描述: [一只橙色的像素小猫，会眨眼]

[生成角色预览]

┌──────────┐
│ [base图] │  ← 用户确认角色外观
└──────────┘
[角色OK，继续生成动画]

Step 2 — 生成动画：
帧数: [4]      每行: [2]
[生成动画] → 进度条逐帧生成 → 预览弹窗 → [确认并保存]
```

流程：
1. 用户输入描述
2. **Step 1** 调用 `generate_base`
3. 显示 base 图预览，用户确认角色外观
4. **Step 2** 逐帧调用 `generate_frame`（显示进度条）
5. 弹出预览弹窗，显示拼接后的 spritesheet 和逐帧小图
6. 确认后：
   - 创建宠物目录（现有 `create_pet`，frame_size 设为 32 作为默认值）
   - 保存 PNG 为 `{petId}/idle.png`
   - 写入 `config.json`：
     ```json
     {
       "animations": {
         "idle": {
           "source": "idle.png",
           "frameTime": 600,
           "loop": true,
           "frameCount": 4,
           "framesPerRow": 2
         }
       },
       "defaultState": "idle",
       "states": {
         "idle": { "entry": "idle", "transitions": {} }
       }
     }
     ```
7. 自动激活新宠物，主窗口立刻显示

### 3. AnimationEditor — 单动画 AI 生成

`AnimationEditor.svelte` 中，每行动画的 "删除" 按钮左侧增加 🎨 按钮：

```
动画列表
idle    idle.png    600    4    2    ☑    —    [🎨] [✕]
walk    walk.png    100    4    2    ☑    —    [🎨] [✕]
```

点击 🎨 弹出 AI 生成弹窗：

```
为动画 "idle" 生成素材

Step 1 — 角色描述（已自动填入宠物名称）：
[一只橙色的像素小猫]
[重新生成角色] [使用当前角色]

Step 2 — 动画参数：
帧数: [4]      每行: [2]
[生成动画] → 进度条 → 预览弹窗 → [确认并保存]
```

流程：
1. 用户确认/修改角色描述（首次生成时需先走 Step 1）
2. 若宠物已有 base 描述缓存，可直接使用（跳过 Step 1）
3. **Step 1** 调用 `generate_base`（首次或用户要求重生成时）
4. **Step 2** 逐帧调用 `generate_frame`（显示进度条）
5. 弹出预览弹窗，用户确认后保存
6. 自动更新当前动画的 `frameCount` / `framesPerRow`
7. 触发 `pet-changed` 事件，主窗口和预览即时更新

## 后端图像后处理

每帧返回前进行后处理：

### 处理流水线

1. **解码** — 将 base64 PNG 解码为 `image::DynamicImage`
2. **背景透明化** — 检测图像四角的平均颜色，将接近该颜色的像素设为透明
3. **颜色量化**（可选）— 限制为 16 色板，增强像素风效果
4. **居中裁剪** — 检测非透明像素边界，裁剪出角色主体区域（去除边缘空白）
5. **编码** — 输出为 base64 PNG（保持 1024×1024 分辨率）

### 透明化算法

```rust
fn make_background_transparent(img: &mut image::RgbaImage, threshold: u8) {
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
        let dist = color_distance(pixel[0], pixel[1], pixel[2], bg_r as u8, bg_g as u8, bg_b as u8);
        if dist < threshold {
            pixel[3] = 0;
        }
    }
}
```

阈值默认 `threshold = 30`。

### 居中裁剪

```rust
fn auto_crop_to_content(img: &image::RgbaImage) -> image::RgbaImage {
    let (min_x, min_y, max_x, max_y) = find_nontransparent_bounds(img);
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;
    img.view(min_x, min_y, w, h).to_image()
}
```

裁剪后角色居中，去除多余空白，拼接时更紧凑。

## 姿势序列

Rust 后端预定义常见动画的姿势序列，用户可覆盖：

```rust
fn get_pose_sequence(animation_name: &str, total_frames: u32) -> Vec<String> {
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
    
    // 循环取姿势直到满足 total_frames
    (0..total_frames)
        .map(|i| default[i as usize % default.len()].to_string())
        .collect()
}
```

姿势序列可随时间扩展，通过配置文件或后续版本更新。

## 前端预览弹窗

因为后端精确拼接，前端预览不再需要网格对齐工具：

```
┌─────────────────────────────────────────┐
│  预览 AI 生成结果                        │
│                                         │
│  ┌─────────────────┐                    │
│  │                 │                    │
│  │   [spritesheet] │                    │
│  │                 │                    │
│  └─────────────────┘                    │
│                                         │
│  逐帧预览:                              │
│  ┌────┬────┬────┬────┐                 │
│  │ 0  │ 1  │ 2  │ 3  │                 │
│  └────┴────┴────┴────┘                 │
│                                         │
│  [重新生成全部]  [重试第2帧]  [取消]  [确认保存]│
└─────────────────────────────────────────┘
```

- **上方大图**：拼接后的完整 spritesheet
- **下方小图**：每帧独立预览，方便用户检查是否有明显不一致的帧
- **重试单帧**：如果某帧明显不对，可以只重试那一帧（调用 `generate_frame` 替换）

## 关键设计决策

### 为什么逐帧生成而非单张 spritesheet

**单张 spritesheet 的痛点**：
- AI 对"4 个姿势在一张图上的网格布局"遵守率约 30-40%
- 帧之间角色颜色、体型漂移严重
- 网格线模糊、偏移，需要用户手动对齐

**逐帧生成的优势**（借鉴 hatch-pet）：
- 每帧独立生成，角色一致性 ~90%+
- 不需要前端手动对齐网格
- 可以单独重试某帧，不浪费其他帧
- 后端精确拼接，100% 对齐

**成本**：
- 4 帧动画：1 (base) + 4 = **5 次 API 调用**
- 8 帧动画：1 + 8 = **9 次 API 调用**
- 但单次成功率接近 100%，总体成本低于反复重试抽奖

### 为什么保持 1024×1024 分辨率

- AI 图像模型只能输出大尺寸（1024×1024）
- 不强制缩放到小尺寸，保留细节
- `SpriteRenderer` 的 `imageSmoothingEnabled = false` 配合 `displayScale` 负责渲染时的像素感
- 用户通过 `displayScale`（1-10）自由控制显示大小

### 为什么是"base 锁定 + 姿势序列"

- base 描述作为角色锚点，防止"每帧角色长得不一样"
- 预定义姿势序列给 AI 明确的每帧动作指引
- 用户可覆盖姿势描述，精细控制动画

## 安全与隐私

| 项目 | 处理 |
|------|------|
| API key 存储 | 仅存 Rust 端 `app-settings.json`，用户数据目录有 OS 文件权限保护 |
| 前端 key 暴露 | 前端不持久化 key，每次从 Rust 命令获取配置 |
| 网络请求 | Rust 后端发起，前端不直接接触 API |
| 日志 | key 在任何日志中脱敏显示为 `sk-...****` |
| URL 验证 | `set_ai_config` 验证 base URL 是合法的 http(s) URL |

## 错误处理

| 场景 | 错误信息 |
|------|----------|
| API key 未配置 | 请先在显示设置中配置 AI API |
| 网络超时 | AI 生成超时，请检查网络或稍后重试 |
| API 返回 401 | API key 无效，请检查配置 |
| API 返回 429 | 请求频率过高，请稍后再试 |
| 内容审核拒绝 | 描述可能违反内容策略，请修改后重试 |
| 某帧生成失败 | Frame 2/4 failed: [具体错误]，其他帧已保留，可重试该帧 |
| 图片格式异常 | 生成的图片格式不支持，请重试 |

## i18n 新增键值

```
ai.title: 'AI Generation'
ai.baseUrl: 'Base URL'
ai.apiKey: 'API Key'
ai.model: 'Model'
ai.save: 'Save'
ai.generate: 'Generate'
ai.generateBase: 'Generate Character'
ai.generateFrame: 'Generate Frame'
ai.regenerateBase: 'Regenerate Character'
ai.useCurrentBase: 'Use Current Character'
ai.description: 'Description'
ai.baseDescription: 'Character Description'
ai.frameCount: 'Frames'
ai.framesPerRow: 'Per Row'
ai.step1: 'Step 1 — Character'
ai.step2: 'Step 2 — Animation'
ai.basePreview: 'Character Preview'
ai.generating: 'Generating...'
ai.generatingFrame: 'Generating frame {current}/{total}...'
ai.success: 'Generated successfully'
ai.emptyPrompt: 'Please enter a description'
ai.noConfig: 'AI not configured. Please set Base URL and API Key in Display Settings.'
ai.baseConfirm: 'Character looks good?'
ai.proceedToStep2: 'Continue to Animation'
ai.retryFrame: 'Retry Frame {index}'
ai.retryAll: 'Regenerate All'
```

## 测试策略

### Rust
- `set_ai_config`：URL 格式验证（合法/非法）
- `get_ai_config`：不返回 key 明文
- `generate_base`：mock HTTP 响应测试成功/错误路径
- `generate_frame`：mock HTTP 响应测试成功/错误路径
- `save_ai_sprite`：多帧拼接测试（不同尺寸、不同数量）
- 后处理流水线：透明化、颜色量化、居中裁剪
- `compose_spritesheet`：边界测试（奇数帧、单帧、大尺寸）

### TypeScript
- `generate_base` / `generate_frame` 命令前端调用封装
- 进度条状态同步（逐帧生成时 UI 更新）
- 配置面板的读写状态同步
- 两步生成弹窗的状态流转（Step 1 → Step 2 进度 → 预览）
- 单帧重试逻辑

## 实现阶段

| 阶段 | 内容 |
|------|------|
| 1 | Rust：`reqwest` + `image` 依赖 + AI 配置存储 + `generate_base` / `generate_frame` 命令 + 后处理 + `save_ai_sprite` 拼接 |
| 2 | 前端：DisplaySettings 新增 AI 配置区域 |
| 3 | 前端：两步 AI 生成弹窗组件（Step 1 base 预览 + Step 2 进度条逐帧生成 + 拼接预览） |
| 4 | 前端：AnimationEditor 新增 🎨 AI 生成按钮 + 单帧重试 |
| 5 | 前端：HomeView 创建弹窗新增 AI 生成选项卡 |
| 6 | 端到端测试 + 文档更新 |
