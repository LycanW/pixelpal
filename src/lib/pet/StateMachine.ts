import type { PetEvent, StateConfig } from './types';
import type { AnimationController } from './AnimationController';

export class StateMachine {
  private currentState: string;

  constructor(
    private readonly states: Record<string, StateConfig>,
    private readonly controller: AnimationController,
    initialState: string,
  ) {
    this.currentState = this.states[initialState] ? initialState : Object.keys(this.states)[0] ?? initialState;
  }

  start() {
    const state = this.states[this.currentState];
    if (state) this.controller.play(state.entry);
  }

  dispatch(event: PetEvent) {
    const transition = this.states[this.currentState]?.transitions[event];
    if (!transition || !this.states[transition.target]) return;

    this.currentState = transition.target;
    this.controller.play(transition.animation ?? this.states[transition.target].entry);
  }
}
