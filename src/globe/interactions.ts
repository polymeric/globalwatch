import * as THREE from 'three';

export interface InteractionState {
  autoSpin: boolean;
  isDragging: boolean;
  previousMouse: { x: number; y: number };
  velocity: { x: number; y: number };
  spinSpeed: number;
}

export function createInteractionState(): InteractionState {
  return {
    autoSpin: true,
    isDragging: false,
    previousMouse: { x: 0, y: 0 },
    velocity: { x: 0, y: 0 },
    spinSpeed: 0.001,
  };
}

const DAMPING = 0.92;
const DRAG_SENSITIVITY = 0.005;
const MAX_POLAR = Math.PI * 0.4; // clamp vertical rotation

export function setupInteractions(
  canvas: HTMLCanvasElement,
  _globe: THREE.Group,
  state: InteractionState,
) {
  const onPointerDown = (e: PointerEvent) => {
    state.isDragging = true;
    state.previousMouse = { x: e.clientX, y: e.clientY };
    state.velocity = { x: 0, y: 0 };
    canvas.style.cursor = 'grabbing';
  };

  const onPointerMove = (e: PointerEvent) => {
    if (!state.isDragging) return;
    const dx = e.clientX - state.previousMouse.x;
    const dy = e.clientY - state.previousMouse.y;
    state.velocity = { x: dx * DRAG_SENSITIVITY, y: dy * DRAG_SENSITIVITY };
    state.previousMouse = { x: e.clientX, y: e.clientY };
  };

  const onPointerUp = () => {
    state.isDragging = false;
    canvas.style.cursor = 'grab';
  };

  const onClick = (_e: MouseEvent) => {
    // Only toggle spin if it wasn't a drag (minimal movement)
    if (Math.abs(state.velocity.x) < 0.001 && Math.abs(state.velocity.y) < 0.001) {
      state.autoSpin = !state.autoSpin;
    }
  };

  canvas.addEventListener('pointerdown', onPointerDown);
  canvas.addEventListener('pointermove', onPointerMove);
  canvas.addEventListener('pointerup', onPointerUp);
  canvas.addEventListener('pointerleave', onPointerUp);
  canvas.addEventListener('click', onClick);
  canvas.style.cursor = 'grab';

  return () => {
    canvas.removeEventListener('pointerdown', onPointerDown);
    canvas.removeEventListener('pointermove', onPointerMove);
    canvas.removeEventListener('pointerup', onPointerUp);
    canvas.removeEventListener('pointerleave', onPointerUp);
    canvas.removeEventListener('click', onClick);
  };
}

export function updateInteractions(globe: THREE.Group, state: InteractionState) {
  if (state.isDragging) {
    globe.rotation.y += state.velocity.x;
    globe.rotation.x += state.velocity.y;
  } else {
    // Apply damping to velocity
    state.velocity.x *= DAMPING;
    state.velocity.y *= DAMPING;
    globe.rotation.y += state.velocity.x;
    globe.rotation.x += state.velocity.y;

    // Auto-spin when idle
    if (state.autoSpin && Math.abs(state.velocity.x) < 0.0001) {
      globe.rotation.y += state.spinSpeed;
    }
  }

  // Clamp polar rotation
  globe.rotation.x = Math.max(-MAX_POLAR, Math.min(MAX_POLAR, globe.rotation.x));
}
