import type { AnimationDef } from './types';
import { FRAMES_PER_ACTION } from './config';
export class AnimationController {
  private animDefs: Record<string, AnimationDef> = {};
  private perFrameDelays: Record<string, number[]> = {};
  currentAction: string = 'idle';
  frameIndex: number = 0;
  private timer: number = 0;
  private activeDuration: number = 0;
  private durationFired: boolean = false;

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
      this.durationFired = false;
    }
  }

  update(deltaMs: number) {
    const def = this.animDefs[this.currentAction];
    if (!def) return;

    this.timer += deltaMs;
    this.activeDuration += deltaMs;

    const delays = this.perFrameDelays[this.currentAction];
    const maxFrame = (def.frameCount ?? (delays ? delays.length : FRAMES_PER_ACTION)) - 1;

    const frameTime = typeof def.frameTime === 'number' ? def.frameTime : NaN;
    if (Number.isNaN(frameTime)) {
      console.error(`[AnimationController] "${this.currentAction}" is missing required field "frameTime"`);
      return;
    }

    let iterations = 0;
    while (iterations < 100) {
      const stepMs = Math.max(1, delays
        ? (frameTime > 0 ? frameTime : delays[this.frameIndex] ?? 100)
        : frameTime);
      if (this.timer < stepMs) break;
      this.timer -= stepMs;
      iterations++;

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

    if (def.duration && this.activeDuration >= def.duration && !this.durationFired) {
      this.durationFired = true;
      this.onActionEnd?.(this.currentAction);
    }
  }
}
