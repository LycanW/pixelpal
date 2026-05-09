# Pet Format Reference

Each pet is a folder under `pets/<name>/` containing at minimum three files:

```text
pets/my-pet/
  manifest.json    # Pet metadata
  config.json      # Animations & state machine
  idle.png         # At least one image
```

## manifest.json

```json
{
  "name": "my-pet",
  "version": "1.0.0",
  "author": "",
  "frameWidth": 32,
  "frameHeight": 32,
  "displayScale": 5,
  "windowWidth": 160,
  "windowHeight": 160
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Display name |
| `version` | string | Semver |
| `author` | string | Optional |
| `frameWidth` | int | Source frame width in pixels |
| `frameHeight` | int | Source frame height in pixels |
| `displayScale` | int | Scale multiplier (1–10) |
| `windowWidth` | int | `frameWidth × displayScale` |
| `windowHeight` | int | `frameHeight × displayScale` |

## config.json

Three top-level keys: `animations`, `defaultState`, `states`.

```json
{
  "animations": { … },
  "defaultState": "idle",
  "states": { … }
}
```

### animations

A map of animation name → definition.

```json
{
  "idle": {
    "source": "idle.png",
    "frameTime": 600,
    "loop": true
  },
  "walk": {
    "source": "walk.gif",
    "frameTime": 0,
    "loop": true
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `source` | string | required | Image filename in the pet folder |
| `frameTime` | int | required | Milliseconds per frame. Set to `0` for GIFs to use per-frame delays. |
| `loop` | bool | required | Restart from frame 0 after last frame? |
| `frameCount` | int | auto | Number of frames. **PNG**: defaults to `4`. **GIF**: auto-detected from file. Set to limit playback. |
| `framesPerRow` | int | `2` | **PNG only.** How many frames per row in the spritesheet. Ignored for GIFs. |
| `duration` | int | — | Max playback time in ms. After this, fires `animation_end`. Useful for non-looping one-shot animations. |

**PNG spritesheet layout**: frames are laid out left-to-right, top-to-bottom. A 2×2 grid with 4 frames at 32×32 each would be a 64×64 PNG:

```
┌───┬───┐
│ 0 │ 1 │
├───┼───┤
│ 2 │ 3 │
└───┴───┘
```

**GIF notes**: When `source` ends with `.gif`:
- `frameTime: 0` → uses the GIF's built-in per-frame delays
- `frameTime: N` (N > 0) → overrides all frames to a uniform delay
- `frameCount` and `framesPerRow` are auto-detected from the GIF

### states

A map of state name → state config. Each state has an entry animation and a set of event-driven transitions.

```json
{
  "idle": {
    "entry": "idle",
    "transitions": {
      "press":   { "target": "walk" },
      "click":   { "target": "react" },
      "dblclick": { "target": "sleep" }
    }
  },
  "walk": {
    "entry": "walk",
    "transitions": {
      "drag_end": { "target": "idle" }
    }
  },
  "react": {
    "entry": "click_reaction",
    "transitions": {
      "animation_end": { "target": "idle" }
    }
  }
}
```

**`entry`**: The animation name to play when entering this state.

**`transitions`**: Map of `event → { target, animation? }`.

| Transition field | Required | Description |
|------------------|----------|-------------|
| `target` | yes | State name to transition to |
| `animation` | no | Override animation to play during this transition. If omitted, uses the target state's `entry`. |

### Events

| Event | Trigger |
|-------|---------|
| `press` | Mouse button pressed on the pet |
| `click` | Single left click |
| `dblclick` | Double left click |
| `right_click` | Right click |
| `drag_start` | Mouse drag begins |
| `drag_end` | Mouse drag ends |
| `animation_end` | Current animation finished (non-looping or duration reached) |

### defaultState

The state the pet starts in when loaded.

```json
"defaultState": "idle"
```

Must match a key in `states`.

## Minimal Example

```json
{
  "animations": {
    "idle": {
      "source": "idle.png",
      "frameTime": 600,
      "loop": true
    }
  },
  "defaultState": "idle",
  "states": {
    "idle": {
      "entry": "idle",
      "transitions": {}
    }
  }
}
```

## Full Example

See `pets/default-cat/config.json` for a complete state machine with idle, walk, react, and sleep states.
