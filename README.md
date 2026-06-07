# PixelPal

[中文](./README_ZH.md)

A pixel-art desktop pet that lives on your screen — always-on-top, transparent, and interactive. Built with Svelte 5 and Tauri v2.

Includes an **AI sprite generator** that creates pixel-art characters and animations from text descriptions using OpenAI-compatible image APIs.

## Features

- **Always-on-top transparent window** — pet walks above other windows, intercepts Alt+F4
- **State machine interactions** — press, click, double-click, drag, right-click trigger configurable animations
- **Multi-format image support** — PNG spritesheets, GIF, WebP, JPEG
- **Multi-pet system** — create, import, and switch pets via tray menu or settings
- **GUI settings** — full editor for animations, image assets, states, and transitions
- **AI sprite generation** — generate base character and animation frames from text prompts
- **i18n** — Chinese and English UI, toggle in settings
- **System autostart** — optional launch on system startup
- **System tray** — show/hide, always-on-top toggle, pet switching, settings access
- **Wayland support** — compositor scale hints, ResizeObserver, drag compatibility

## Quick Start

```sh
npm install
npx tauri dev
```

The pet appears at the bottom-right corner of your primary screen.

| Command | Description |
|---------|-------------|
| `npm run dev` | Frontend only (Vite dev server) |
| `npm run build` | Frontend production build |
| `npm run check` | TypeScript + Svelte type checking |
| `npm test` | Run Vitest frontend tests |
| `npx tauri dev` | Full app in development mode |
| `npx tauri build` | Production build → `src-tauri/target/release/` |
| `python scripts/generate-sprites.py` | Generate default pet sprites |

## Requirements

| Tool | Version |
|------|---------|
| Node.js | 18+ |
| Rust | 1.77.2+ (stable, edition 2021) |
| OS | Windows 10+ / macOS 12+ / Linux (X11 + Wayland) |

See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for platform-specific system libraries.

## Pet Format

Pets are stored in the OS data directory (`~/.local/share/pixelpal/pets/` on Linux). Each pet is a directory containing:

```
pets/my-pet/
  manifest.json   # name, frame size, display scale
  config.json     # animations, states, transitions
  idle.png        # spritesheet: configurable grid (e.g. 2×2)
  walk.gif        # or animated GIF / WebP / JPEG
```

The `config.json` defines:
- **animations** — named animation defs (source file, frame time, frame count, frames per row, loop)
- **states** — named states with an entry animation and event→transition mappings
- **defaultState** — which state the pet starts in

For the full spec, see [PET_FORMAT.md](PET_FORMAT.md).

### Creating Pets

1. Open Settings via the tray menu
2. Click **+ New** — creates an empty pet directory with `manifest.json`
3. Click the gear icon to open the pet editor
4. **Config** tab — set name, frame size, display scale
5. **Animations** tab — import images (PNG, GIF, WebP, JPEG), create animations
6. **Interactions** tab — create states and define transitions for mouse events

### AI Generation

1. Configure your API endpoint in **Settings → Display → AI Configuration** (OpenAI-compatible, defaults to `gpt-image-1`)
2. From the pet home screen, click **+ New** and select **Generate with AI**
3. **Step 1**: Describe your character or use an existing `base.png`
4. **Step 2**: Configure frame count (1–16) and frames per row (1–8)
5. **Step 3**: Preview the spritesheet and save to the pet directory

## Architecture

Two independent Tauri windows, each with its own Vite entry point:

**Main Window** (`index.html`):
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

**Settings Window** (`settings.html`):
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

**Backend** (Rust, 25 Tauri commands):
```
┌─────────────────────────────────────────────────────────┐
│ commands.rs          │ pet I/O, settings, import/export  │
│ ai_commands.rs       │ AI config, image generation       │
│ ai_image.rs          │ transparency, crop, spritesheet   │
│ ai_prompts.rs        │ prompt engineering, pose sequences │
│ lib.rs               │ tray menu, window mgmt, autostart │
└─────────────────────────────────────────────────────────┘
```

## Testing

```sh
npm test                  # Frontend: Vitest (AnimationController + StateMachine)
cargo test -p pixelpal    # Backend: 23 Rust unit tests (commands, AI, image processing)
```

## Distribution

```sh
npx tauri build
# Linux:
#   → src-tauri/target/release/bundle/deb/PixelPal_*_amd64.deb
#   → src-tauri/target/release/bundle/rpm/PixelPal-*.x86_64.rpm
# Windows:
#   → src-tauri/target/release/bundle/nsis/PixelPal_*_x64-setup.exe
#   → src-tauri/target/release/pixelpal.exe  (portable)
```

## License

MIT
