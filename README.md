# PixelPal

A pixel-art desktop pet that lives on your screen. Built with Svelte 5 and Tauri v2.

## Project Structure

```text
pixelpal/
  src/                          # Frontend (Svelte 5 + TypeScript)
    main.ts                     #   Main window entry
    App.svelte                  #   Root component
    lib/pet/                    #   Pet engine
      PetCanvas.svelte          #     Transparent canvas, input handling
      AnimationController.ts    #     Frame timing & playback
      StateMachine.ts           #     State transitions
      SpriteLoader.ts           #     PNG/GIF loading
      SpriteRenderer.ts         #     Canvas rendering
      types.ts                  #     Shared type definitions
    settings/                   #   Settings window (multi-page)
      HomeView.svelte           #     Pet list & creation
      PetDetailView.svelte      #     Animations / Interactions / Config tabs
      AnimationEditor.svelte    #     Manage animations & image assets
      StateEditor.svelte        #     State machine editor
      PetCard.svelte            #     Animated preview card
      DisplaySettings.svelte    #     Pet directory & scale
  src-tauri/                    # Backend (Rust + Tauri v2)
    src/
      main.rs                   #   Entry point
      lib.rs                    #   Tray menu, command registration
      commands.rs               #   File I/O, pet scanning, window management
    tauri.conf.json             #   Window config, CSP, bundle settings
    Cargo.toml                  #   Rust dependencies
  settings.html                 # Settings window entry point
  index.html                    # Main window entry point
```

## Environment

| Requirement | Minimum |
|-------------|---------|
| [Node.js](https://nodejs.org/) | 18+ |
| [Rust](https://rustup.rs/) | 1.70+ (stable) |
| OS | Windows 10+ / macOS 12+ / Linux |

Tauri v2 requires platform-specific system libraries. See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

## Quick Start

```sh
npm install
npx tauri dev
```

- **`npm run dev`** — frontend only (Vite dev server)
- **`npm run build`** — frontend production build
- **`npm run check`** — TypeScript + Svelte type checking
- **`npx tauri build`** — full production build

## Features

- **Transparent always-on-top window** — pet walks above other windows
- **State machine interactions** — press, click, double-click, drag, right-click
- **PNG spritesheet** and **GIF** animation support
- **Multi-pet system** — switch pets via tray menu, each with independent config
- **GUI settings** — manage animations, image assets, and interaction logic without editing JSON by hand
- **System tray** — hide/show, always-on-top toggle, pet switching

For details on the config.json format and manual pet authoring, see [PET_FORMAT.md](PET_FORMAT.md).
