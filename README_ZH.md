# PixelPal

[English](./README.md)

一个常驻屏幕的像素风桌面宠物——始终置顶、透明窗口、可交互。基于 Svelte 5 和 Tauri v2 构建。

内置 **AI 精灵生成器**，通过 OpenAI 兼容的图像 API，根据文字描述生成像素风角色和动画。

## 功能特性

- **始终置顶透明窗口** — 宠物在所有窗口之上，拦截 Alt+F4 防止误关
- **状态机交互** — 按压、单击、双击、拖拽、右键可触发可配置的动画
- **多格式图像支持** — PNG 精灵表、GIF、WebP、JPEG
- **多宠物系统** — 通过托盘菜单或设置界面创建、导入、切换宠物
- **图形化设置** — 完整的动画、图像素材、状态和过渡编辑器
- **AI 精灵生成** — 根据文字提示生成基础角色和动画帧
- **国际化** — 中文和英文界面，设置中一键切换
- **系统自启动** — 可选择开机自动启动
- **系统托盘** — 显示/隐藏、置顶切换、宠物切换、设置入口
- **Wayland 支持** — 合成器缩放提示、ResizeObserver、拖拽兼容

## 快速开始

```sh
npm install
npx tauri dev
```

宠物会出现在主屏幕右下角。

| 命令 | 说明 |
|---------|-------------|
| `npm run dev` | 仅前端（Vite 开发服务器） |
| `npm run build` | 前端生产构建 |
| `npm run check` | TypeScript + Svelte 类型检查 |
| `npm test` | 运行 Vitest 前端测试 |
| `npx tauri dev` | 完整应用开发模式 |
| `npx tauri build` | 生产构建 → `src-tauri/target/release/` |
| `python scripts/generate-sprites.py` | 生成默认宠物精灵图 |

## 环境要求

| 工具 | 版本 |
|------|---------|
| Node.js | 18+ |
| Rust | 1.77.2+ (stable, edition 2021) |
| 操作系统 | Windows 10+ / macOS 12+ / Linux（X11 + Wayland） |

各平台所需系统库请参见 [Tauri 前置条件](https://v2.tauri.app/start/prerequisites/)。

## 宠物格式

宠物数据存储在系统数据目录中（Linux 下为 `~/.local/share/pixelpal/pets/`）。每个宠物是一个目录，包含：

```
pets/my-pet/
  manifest.json   # 名称、帧尺寸、显示缩放
  config.json     # 动画、状态、过渡定义
  idle.png        # 精灵表：可配置网格（如 2×2）
  walk.gif        # 或动画 GIF / WebP / JPEG
```

`config.json` 定义：
- **animations** — 具名动画定义（源文件、帧时长、帧数、每行帧数、循环）
- **states** — 具名状态，包含入场动画和事件→过渡映射
- **defaultState** — 宠物的初始状态

完整规范请参阅 [PET_FORMAT.md](PET_FORMAT.md)。

### 创建宠物

1. 通过托盘菜单打开设置
2. 点击 **+ 新建** — 创建空的宠物目录和 `manifest.json`
3. 点击齿轮图标打开宠物编辑器
4. **配置** 选项卡 — 设置名称、帧尺寸、显示缩放
5. **动画** 选项卡 — 导入图像（PNG、GIF、WebP、JPEG），创建动画
6. **交互** 选项卡 — 创建状态并定义鼠标事件过渡

### AI 生成

1. 在 **设置 → 显示 → AI 配置** 中配置 API 端点（兼容 OpenAI，默认模型 `gpt-image-1`）
2. 在宠物主页点击 **+ 新建**，选择 **AI 生成**
3. **第 1 步**：描述你的角色，或使用已有的 `base.png`
4. **第 2 步**：配置帧数（1–16）和每行帧数（1–8）
5. **第 3 步**：预览精灵表并保存到宠物目录

## 架构

两个独立的 Tauri 窗口，各自对应一个 Vite 入口：

**主窗口** (`index.html`)：
```
┌─────────────────────┐
│ App.svelte           │
│  └─ PetCanvas.svelte │
│      ├─ Animation    │
│      ├─ StateMachine │
│      ├─ SpriteLoader │
│      ├─ SpriteRender │
│      └─ i18n         │
└─────────────────────┘
```

**设置窗口** (`settings.html`)：
```
┌──────────────────────────┐
│ Settings.svelte           │
│  ├─ HomeView.svelte       │
│  │   ├─ PetCard.svelte    │
│  │   └─ AiGenerationModal │
│  ├─ PetDetailView.svelte  │
│  │   ├─ PetConfigEditor   │
│  │   ├─ AnimationEditor   │
│  │   └─ StateEditor       │
│  ├─ DisplaySettings.svelte│
│  │   └─ AiConfigPanel     │
│  └─ Toast.svelte          │
└──────────────────────────┘
```

**后端**（Rust，25 个 Tauri 命令）：
```
┌─────────────────────────────────────────────────────────┐
│ commands.rs          │ 宠物 IO、设置、导入/导出           │
│ ai_commands.rs       │ AI 配置、图像生成                  │
│ ai_image.rs          │ 透明化、裁剪、精灵表合成            │
│ ai_prompts.rs        │ 提示词工程、动作序列                │
│ lib.rs               │ 托盘菜单、窗口管理、自启动          │
└─────────────────────────────────────────────────────────┘
```

## 测试

```sh
npm test                  # 前端：Vitest（AnimationController + StateMachine）
cargo test -p pixelpal    # 后端：23 个 Rust 单元测试（命令、AI、图像处理）
```

## 分发构建

```sh
npx tauri build
# Linux:
#   → src-tauri/target/release/bundle/deb/PixelPal_*_amd64.deb
#   → src-tauri/target/release/bundle/rpm/PixelPal-*.x86_64.rpm
# Windows:
#   → src-tauri/target/release/bundle/nsis/PixelPal_*_x64-setup.exe
#   → src-tauri/target/release/pixelpal.exe（免安装）
```

## 许可证

MIT
