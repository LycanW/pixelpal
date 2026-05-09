export type PetEvent = 'press' | 'drag_start' | 'drag_end' | 'click' | 'dblclick' | 'right_click' | 'animation_end';

export interface GifFrameData {
  frames: ImageBitmap[];
  delays: number[];
}

export interface AnimationDef {
  source: string;
  frameTime: number;
  loop: boolean;
  duration?: number;
  frameCount?: number;
  framesPerRow?: number;
}

export interface Transition {
  target: string;
  animation?: string;
}

export interface StateConfig {
  entry: string;
  transitions: Partial<Record<PetEvent, Transition>>;
}

export interface PetConfig {
  animations: Record<string, AnimationDef>;
  defaultState: string;
  states: Record<string, StateConfig>;
}
