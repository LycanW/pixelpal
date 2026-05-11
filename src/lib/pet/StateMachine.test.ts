import { describe, it, expect } from 'vitest';
import { StateMachine } from './StateMachine';
import { AnimationController } from './AnimationController';

function createMockController(): AnimationController {
  const ctrl = new AnimationController();
  ctrl.setAnimations({
    idle: { source: 'idle.png', frameTime: 100, loop: true },
    walk: { source: 'walk.png', frameTime: 100, loop: true },
    react: { source: 'react.png', frameTime: 100, loop: false },
    sleep: { source: 'sleep.png', frameTime: 100, loop: true },
  });
  return ctrl;
}

describe('StateMachine', () => {
  it('starts with default state entry animation', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: { entry: 'idle', transitions: {} },
      walk: { entry: 'walk', transitions: {} },
    }, ctrl, 'idle');

    sm.start();
    expect(ctrl.currentAction).toBe('idle');
  });

  it('falls back to first available state if initial state is invalid', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      walk: { entry: 'walk', transitions: {} },
      idle: { entry: 'idle', transitions: {} },
    }, ctrl, 'nonexistent');

    sm.start();
    expect(ctrl.currentAction).toBe('walk');
  });

  it('transitions on event and plays target entry animation', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: { entry: 'idle', transitions: { press: { target: 'walk' } } },
      walk: { entry: 'walk', transitions: {} },
    }, ctrl, 'idle');

    sm.start();
    expect(ctrl.currentAction).toBe('idle');

    sm.dispatch('press');
    expect(ctrl.currentAction).toBe('walk');
  });

  it('plays override animation if specified in transition', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: { entry: 'idle', transitions: { click: { target: 'walk', animation: 'react' } } },
      walk: { entry: 'walk', transitions: {} },
    }, ctrl, 'idle');

    sm.start();
    sm.dispatch('click');
    expect(ctrl.currentAction).toBe('react');
  });

  it('ignores undefined events', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: { entry: 'idle', transitions: {} },
    }, ctrl, 'idle');

    sm.start();
    sm.dispatch('press');
    expect(ctrl.currentAction).toBe('idle');
  });

  it('ignores transitions to nonexistent states', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: { entry: 'idle', transitions: { press: { target: 'ghost' } } },
    }, ctrl, 'idle');

    sm.start();
    sm.dispatch('press');
    expect(ctrl.currentAction).toBe('idle');
  });

  it('handles complex state machine like default-cat', () => {
    const ctrl = createMockController();
    const sm = new StateMachine({
      idle: {
        entry: 'idle',
        transitions: {
          press: { target: 'walk' },
          click: { target: 'react' },
          dblclick: { target: 'sleep' },
        },
      },
      walk: {
        entry: 'walk',
        transitions: {
          drag_end: { target: 'idle' },
          click: { target: 'react' },
        },
      },
      react: {
        entry: 'react',
        transitions: {
          animation_end: { target: 'idle' },
          drag_start: { target: 'walk' },
        },
      },
      sleep: {
        entry: 'sleep',
        transitions: {
          click: { target: 'idle' },
          press: { target: 'idle' },
        },
      },
    }, ctrl, 'idle');

    sm.start();
    expect(ctrl.currentAction).toBe('idle');

    sm.dispatch('press');
    expect(ctrl.currentAction).toBe('walk');

    sm.dispatch('click');
    expect(ctrl.currentAction).toBe('react');

    sm.dispatch('animation_end');
    expect(ctrl.currentAction).toBe('idle');

    sm.dispatch('dblclick');
    expect(ctrl.currentAction).toBe('sleep');

    sm.dispatch('press');
    expect(ctrl.currentAction).toBe('idle');
  });
});
