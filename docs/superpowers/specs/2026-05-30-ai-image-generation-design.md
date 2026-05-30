# PixelPal AI 图像生成设计文档

## 目标

在 PixelPal 中集成 AI 图像生成功能，降低宠物创作门槛。用户自行提供 OpenAI 兼容的 API base URL 和 API key，系统根据描述自动生成像素风角色素材，自动适配现有动画系统。

## 核心设计原则

- **渐进增强**：AI 生成是现有工作流的补充，不替代手动创作
- **两种模式**：创建时一键生成（快速体验）+ 编辑时单动画生成（精细控制）
- **最小侵入**：复用现有 `AnimationController`、`SpriteRenderer` 和 `config.json` 格式
- **安全优先**：API key 只存 Rust 后端，前端不持久化密钥

## 架构概述

```
用户描述角色
    │
    ▼
前端弹窗（prompt + frameCount + framesPerRow + frameSize）
    │
    ▼
Rust 命令 generate_sprite(prompt, frame_count, frames_per_row, frame_size)
    │
    ▼
HTTP POST {base_url}/v1/images/generations
    │
    ▼
AI 返回图片 URL / base64
    │
    ▼
Rust 下载图片
    │
    ▼
后端图像后处理（缩放、透明化、网格对齐）
    │
    ▼
返回 base64 PNG data URL
    │
    ▼
前端预览弹窗（可微调裁剪区域、网格偏移）
    │
    ▼
用户确认后保存为 PNG 到宠物目录
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
| `generate_sprite` | `{ prompt: String, frame_count: u32, frames_per_row: u32, frame_size: u32 }` | `Result<String, String>` | 返回后处理后的 base64 PNG data URL |
| `save_ai_sprite` | `{ pet_id: String, filename: String, base64: String, crop_x: u32, crop_y: u32, crop_w: u32, crop_h: u32 }` | `Result<(), String>` | 按裁剪参数截取并保存 PNG 到宠物目录 |

#### `generate_sprite` 实现细节

1. **构建请求体**（OpenAI images/generations 格式）：
   ```json
   {
     "model": "gpt-image-1",
     "prompt": "A cute pixel-art {description}, arranged in a {rows}x{cols} sprite sheet grid, transparent background, crisp pixel edges, {frame_size}x{frame_size} pixels per frame",
     "size": "1024x1024",
     "quality": "high",
     "n": 1
   }
   ```
   
2. **发送 HTTP 请求**（使用 `reqwest` crate），超时 60 秒
3. **解析响应**：提取 `data[0].b64_json` 或 `data[0].url`
4. **图像后处理**（见下方"后端图像后处理"章节）
5. **返回 base64 data URL**（`data:image/png;base64,...`）

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

AI 生成模式：
描述: [一只橙色的像素小猫，会眨眼]
帧数: [4]      每行: [2]      (自动生成 2x2 spritesheet)

[生成并创建]
```

流程：
1. 用户输入描述，确认帧数和行列数
2. 调用 `generate_sprite`
3. 图片返回后：
   - 创建宠物目录（现有 `create_pet`）
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
4. 自动激活新宠物，主窗口立刻显示

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
描述: [这只小猫 idle 状态的动画]
帧数: [4]      每行: [2]
[生成] [取消]
```

流程：
1. 用户确认/修改描述
2. 调用 `generate_sprite`
3. 图片返回后保存为 PNG（若已有同名文件则覆盖）
4. 自动更新当前动画的 `frameCount` / `framesPerRow`
5. 触发 `pet-changed` 事件，主窗口和预览即时更新

## 后端图像后处理

AI 生成的图片不保证精确遵守网格约束。Rust 后端在返回前进行自动修正：

### 处理流水线

1. **解码** — 将 base64/下载的 PNG 解码为 `image::DynamicImage`
2. **缩放** — 缩放到 `frame_size × frames_per_row` 的精确目标尺寸
3. **背景透明化** — 检测图像四角的平均颜色，将接近该颜色的像素设为透明（处理 AI 生成的纯色背景）
4. **颜色量化**（可选）— 限制为 16 色板，增强像素风效果
5. **编码** — 输出为 base64 PNG

### 透明化算法

```rust
fn make_background_transparent(img: &mut image::RgbaImage, threshold: u8) {
    // 采样四角颜色作为背景参考色
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
            pixel[3] = 0; // alpha = 0
        }
    }
}
```

阈值默认 `threshold = 30`（0-255 范围内），可通过参数调整。

## 前端预览与微调

AI 生成后**不直接保存**，而是弹出预览弹窗让用户确认：

### 预览弹窗 UI

```
┌─────────────────────────────────────────┐
│  预览 AI 生成结果                        │
│                                         │
│  ┌─────────────────┐                    │
│  │                 │  裁剪区域          │
│  │   [预览图]      │  X: [0    ] px     │
│  │                 │  Y: [0    ] px     │
│  │   叠加网格线     │  W: [64   ] px     │
│  │                 │  H: [64   ] px     │
│  └─────────────────┘                    │
│                                         │
│  帧数: [4]    每行: [2]                 │
│                                         │
│  实时帧预览:                            │
│  ┌──┬──┐                               │
│  │ 0│ 1│                               │
│  ├──┼──┤                               │
│  │ 2│ 3│                               │
│  └──┴──┘                               │
│                                         │
│  [重新生成]    [取消]    [确认并保存]     │
└─────────────────────────────────────────┘
```

### 交互逻辑

- **预览图**：显示完整的后处理图片，叠加半透明白色网格线（`2x2` / `4x2` 等）
- **裁剪区域**：调整 X/Y/W/H，实时更新预览（防止 AI 生成的图有边框或偏移）
- **帧数/每行**：调整时重新计算网格线并更新帧预览
- **实时帧预览**：下方小图展示切割后的每一帧效果
- **重新生成**：回到描述输入，保留参数
- **确认并保存**：
  - 调用 Rust 命令 `save_ai_sprite(pet_id, filename, base64, crop_x, crop_y, crop_w, crop_h)`
  - Rust 后端按裁剪参数截取图像、保存为 PNG
  - 前端更新 `config.json` 动画定义

### 裁剪参数默认值

Rust 后端后处理后，前端默认裁剪参数为：
- `crop_x = 0, crop_y = 0`
- `crop_w = frame_size * frames_per_row`
- `crop_h = frame_size * ceil(frame_count / frames_per_row)`

## 关键设计决策

### 为什么是"后处理 + 预览"而非纯 prompt

AI 图像生成对精确几何约束的遵守率约 70-80%：
- 背景可能带轻微颜色而非完全透明
- 网格可能有 2-5px 的偏移
- 帧之间可能有细边框

后处理解决 80% 的问题，预览微调覆盖剩余的 20%。这比反复重试更省 token 和时间。

### 不需要"切割"

PixelPal 的渲染系统原生支持 spritesheet：
- `SpriteRenderer.ts` 的 `drawFrame` 通过 `frameIndex % framesPerRow` 和 `Math.floor(frameIndex / framesPerRow)` 定位帧
- 后处理确保图像精确匹配目标尺寸后，整张 spritesheet 直接保存
- 配置正确的 `frameCount` 和 `framesPerRow` 即可正确渲染

### 默认尺寸策略

| 帧尺寸 | 总尺寸 | 说明 |
|--------|--------|------|
| 32x32 | 64x64 (2x2) | 默认，兼容现有 default-cat |
| 32x32 | 128x64 (4x2) | 4 帧长动画 |
| 64x64 | 128x128 (2x2) | 高分辨率 |

AI 生成统一请求 `1024x1024`，由 `SpriteRenderer` 按比例缩放渲染（`imageSmoothingEnabled = false` 保持像素感）。

### Prompt 模板

```
A pixel-art character sprite sheet for "{animation_name}" state: {user_description}.
Arranged in a {rows}x{cols} grid of {frame_w}x{frame_h} pixel frames.
Transparent background. Crisp clean pixel edges. Game-asset style.
```

示例（idle 动画）：
```
A pixel-art character sprite sheet for "idle" state: a cute orange cat blinking slowly.
Arranged in a 2x2 grid of 32x32 pixel frames.
Transparent background. Crisp clean pixel edges. Game-asset style.
```

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
| 图片格式异常 | 生成的图片格式不支持，请重试 |

## i18n 新增键值

```
ai.title: 'AI Generation'
ai.baseUrl: 'Base URL'
ai.apiKey: 'API Key'
ai.model: 'Model'
ai.save: 'Save'
ai.generate: 'Generate'
ai.description: 'Description'
ai.frameCount: 'Frames'
ai.framesPerRow: 'Per Row'
ai.generating: 'Generating...'
ai.success: 'Generated successfully'
ai.emptyPrompt: 'Please enter a description'
ai.noConfig: 'AI not configured. Please set Base URL and API Key in Display Settings.'
```

## 测试策略

### Rust
- `set_ai_config`：URL 格式验证（合法/非法）
- `get_ai_config`：不返回 key 明文
- `generate_sprite`：mock HTTP 响应测试成功/错误路径

### TypeScript
- `generate_sprite` 命令前端调用封装
- 配置面板的读写状态同步
- 生成弹窗的表单验证

## 实现阶段

| 阶段 | 内容 |
|------|------|
| 1 | Rust：`reqwest` + `image` 依赖 + AI 配置存储 + `generate_sprite` 命令 + 图像后处理 |
| 2 | 前端：DisplaySettings 新增 AI 配置区域 |
| 3 | 前端：AI 预览弹窗组件（裁剪 + 帧预览） |
| 4 | 前端：AnimationEditor 新增 🎨 AI 生成按钮 |
| 5 | 前端：HomeView 创建弹窗新增 AI 生成选项卡 |
| 6 | 端到端测试 + 文档更新 |
