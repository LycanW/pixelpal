import { describe, it, expect, vi } from 'vitest';
import { AnimationController } from './AnimationController';

describe('AnimationController', () => {
  it('starts at frame 0 with default action', () => {
    const ctrl = new AnimationController();
    expect(ctrl.currentAction).toBe('idle');
    expect(ctrl.frameIndex).toBe(0);
  });

  it('play switches action and resets frame/timer', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      walk: { source: 'walk.png', frameTime: 100, loop: true },
    });

    ctrl.play('walk');
    expect(ctrl.currentAction).toBe('walk');
    expect(ctrl.frameIndex).toBe(0);
  });

  it('does not reset when playing same action', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      idle: { source: 'idle.png', frameTime: 100, loop: true },
    });

    ctrl.update(150);
    expect(ctrl.frameIndex).toBe(1);

    ctrl.play('idle');
    expect(ctrl.frameIndex).toBe(1);
  });

  it('advances frames based on frameTime', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 100, loop: true },
    });
    ctrl.play('anim');

    expect(ctrl.frameIndex).toBe(0);
    ctrl.update(50);
    expect(ctrl.frameIndex).toBe(0);
    ctrl.update(50);
    expect(ctrl.frameIndex).toBe(1);
    ctrl.update(250);
    expect(ctrl.frameIndex).toBe(3);
  });

  it('loops back to frame 0 for looping animation', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 100, loop: true, frameCount: 2 },
    });
    ctrl.play('anim');

    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(1);
    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(0);
  });

  it('calls onEnd and stops for non-looping animation', () => {
    const ctrl = new AnimationController();
    const onEnd = vi.fn();
    ctrl.onEnd(onEnd);

    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 100, loop: false, frameCount: 2 },
    });
    ctrl.play('anim');

    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(1);
    expect(onEnd).not.toHaveBeenCalled();

    ctrl.update(100);
    expect(onEnd).toHaveBeenCalledWith('anim');
  });

  it('uses per-frame delays when provided', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 0, loop: true },
    });
    ctrl.setFrameDelays('anim', [50, 150, 50]);
    ctrl.play('anim');

    ctrl.update(50);
    expect(ctrl.frameIndex).toBe(1);
    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(1);
    ctrl.update(50);
    expect(ctrl.frameIndex).toBe(2);
  });

  it('uses frameCount override if provided', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 100, loop: true, frameCount: 2 },
    });
    ctrl.play('anim');

    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(1);
    ctrl.update(100);
    expect(ctrl.frameIndex).toBe(0);
  });

  it('triggers onEnd via duration limit', () => {
    const ctrl = new AnimationController();
    const onEnd = vi.fn();
    ctrl.onEnd(onEnd);

    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 100, loop: true, duration: 250 },
    });
    ctrl.play('anim');

    ctrl.update(200);
    expect(onEnd).not.toHaveBeenCalled();
    ctrl.update(100);
    expect(onEnd).toHaveBeenCalledWith('anim');
  });

  it('caps iterations to prevent infinite loops', () => {
    const ctrl = new AnimationController();
    ctrl.setAnimations({
      anim: { source: 'a.png', frameTime: 1, loop: true },
    });
    ctrl.play('anim');

    ctrl.update(1000);
    // 100 iterations cap prevents runaway; after 100 advances + wraps, frame resets
    expect(ctrl.frameIndex).toBe(0);
  });
});
