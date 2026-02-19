import * as THREE from 'three';

export interface InteractionState {
  autoSpin: boolean;
  isDragging: boolean;
  previousMouse: { x: number; y: number };
  velocity: { x: number; y: number };
  spinSpeed: number;
}

export type HoverData = {
  label: string;
  headline: string;
  severity: string;
  category: string;
  x: number;
  y: number;
} | null;

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
  camera?: THREE.Camera,
  markerGroup?: THREE.Group,
  onHover?: (data: HoverData) => void,
) {
  const raycaster = new THREE.Raycaster();

  const onPointerDown = (e: PointerEvent) => {
    state.isDragging = true;
    state.previousMouse = { x: e.clientX, y: e.clientY };
    state.velocity = { x: 0, y: 0 };
    canvas.style.cursor = 'grabbing';
    onHover?.(null);
  };

  const onPointerMove = (e: PointerEvent) => {
    // Hover raycasting (only when not dragging)
    if (!state.isDragging && camera && markerGroup && onHover) {
      const rect = canvas.getBoundingClientRect();
      const ndcX = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      const ndcY = -((e.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(new THREE.Vector2(ndcX, ndcY), camera);
      const hits = raycaster.intersectObjects(markerGroup.children, false);
      if (hits.length > 0) {
        const { userData } = hits[0].object;
        onHover({ label: userData.label, headline: userData.headline, severity: userData.severity, category: userData.category, x: e.clientX, y: e.clientY });
      } else {
        onHover(null);
      }
    }

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

  const onPointerLeave = () => {
    state.isDragging = false;
    canvas.style.cursor = 'grab';
    onHover?.(null);
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
  canvas.addEventListener('pointerleave', onPointerLeave);
  canvas.addEventListener('click', onClick);
  canvas.style.cursor = 'grab';

  return () => {
    canvas.removeEventListener('pointerdown', onPointerDown);
    canvas.removeEventListener('pointermove', onPointerMove);
    canvas.removeEventListener('pointerup', onPointerUp);
    canvas.removeEventListener('pointerleave', onPointerLeave);
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
