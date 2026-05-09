import type { AnimationDef } from './types';
import { FRAMES_PER_ACTION } from './config';
export class AnimationController {
  private animDefs: Record<string, AnimationDef> = {};
  private perFrameDelays: Record<string, number[]> = {};
  currentAction: string = 'idle';
  frameIndex: number = 0;
  private timer: number = 0;
  private activeDuration: number = 0;

  private onActionEnd: ((action: string) => void) | null = null;

  setAnimations(defs: Record<string, AnimationDef>) {
    this.animDefs = defs;
  }

  setFrameDelays(animName: string, delays: number[]) {
    this.perFrameDelays[animName] = delays;
  }

  onEnd(cb: (action: string) => void) {
    this.onActionEnd = cb;
  }

  play(action: string) {
    if (this.currentAction !== action) {
      this.currentAction = action;
      this.frameIndex = 0;
      this.timer = 0;
      this.activeDuration = 0;
    }
  }

  update(deltaMs: number) {
    const def = this.animDefs[this.currentAction];
    if (!def) return;

    this.timer += deltaMs;
    this.activeDuration += deltaMs;

    const delays = this.perFrameDelays[this.currentAction];
    // For GIF: use per-frame delay from the file; frameTime in def can override uniformly
    const frameTime = delays
      ? (def.frameTime > 0 ? def.frameTime : delays[this.frameIndex] ?? 100)
      : def.frameTime;
    const maxFrame = (def.frameCount ?? (delays ? delays.length : FRAMES_PER_ACTION)) - 1;

    if (this.timer >= frameTime) {
      this.timer -= frameTime;

      if (this.frameIndex >= maxFrame) {
        if (def.loop) {
          this.frameIndex = 0;
        } else {
          this.onActionEnd?.(this.currentAction);
          return;
        }
      } else {
        this.frameIndex++;
      }
    }

    // Check duration *after* frame advance so the final frame is shown
    if (def.duration && this.activeDuration >= def.duration) {
      this.onActionEnd?.(this.currentAction);
    }
  }
}
