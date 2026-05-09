# PixelPal

A pixel-art desktop pet that lives on your screen — always-on-top, transparent, and interactive. Built with Svelte 5 and Tauri v2.

## Features

- **Always-on-top transparent window** — pet walks above other windows
- **State machine interactions** — press, click, double-click, drag, right-click trigger configurable animations
- **PNG spritesheet & GIF support** — 2×2 grid spritesheets or animated GIFs
- **Multi-pet system** — create, import, and switch pets via tray menu or settings
- **GUI settings** — full editor for animations, image assets, and interaction states
- **System tray** — show/hide, always-on-top toggle, pet switching

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
| `npx tauri dev` | Full app in development mode |
| `npx tauri build` | Production build → `src-tauri/target/release/` |

## Requirements

| Tool | Version |
|------|---------|
| Node.js | 18+ |
| Rust | 1.70+ (stable) |
| OS | Windows 10+ / macOS 12+ / Linux |

See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for platform-specific system libraries.

## Pet Format

Each pet is a directory under `pets/` (next to the executable) containing:

```
pets/my-pet/
  manifest.json   # name, frame size, display scale
  config.json     # animations, states, transitions
  idle.png        # spritesheet: 2×2 grid, each cell one frame
  walk.gif        # or animated GIF
```

The `config.json` defines:
- **animations** — named animation defs (source file, frame time, frame count, loop)
- **states** — named states with an entry animation and event→transition mappings
- **defaultState** — which state the pet starts in

For the full spec, see [PET_FORMAT.md](PET_FORMAT.md).

### Creating Pets

1. Open Settings via the tray menu
2. Click **+ New** — this creates an empty pet directory with `manifest.json`
3. Click the gear icon to open the pet editor
4. **Config** tab — set name, frame size, display scale
5. **Animations** tab — import images, create animations
   - PNG: 2×2 spritesheet recommended. Frames must be divisible by Per Row.
   - GIF: all frames extracted automatically
6. **Interactions** tab — create states and define transitions for mouse events

## Architecture

```
Frontend (Svelte 5)          Backend (Rust + Tauri v2)
┌─────────────────────┐     ┌──────────────────────┐
│ App.svelte           │     │ lib.rs               │
│  └─ PetCanvas.svelte │────▶│  └─ commands.rs      │
│      ├─ Animation    │ IPC │      ├─ read_json    │
│      ├─ StateMachine │     │      ├─ read_sprite  │
│      ├─ SpriteLoader │     │      ├─ list_pets    │
│      └─ SpriteRender │     │      └─ create_pet   │
│                      │     │  └─ tray menu        │
│ Settings.svelte      │     │  └─ window mgmt      │
│  ├─ HomeView         │────▶│                      │
│  ├─ AnimationEditor  │ IPC │                      │
│  ├─ StateEditor      │     │                      │
│  └─ DisplaySettings  │     │                      │
└─────────────────────┘     └──────────────────────┘
```

## Distribution

The release build produces a standalone executable — no installer needed. Copy `pixelpal.exe` + `pets/` folder to any machine with Windows 10+ (WebView2 is included with the OS).

```
npx tauri build
# → src-tauri/target/release/bundle/nsis/PixelPal_*_x64-setup.exe
# → src-tauri/target/release/pixelpal.exe  (portable)
```

## License

MIT
