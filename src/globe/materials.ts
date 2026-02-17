import * as THREE from 'three';

// Color palette — green-on-black terminal aesthetic
export const COLORS = {
  background: 0x0a1a0a,
  primary: 0x4af060,
  bright: 0x7effa7,
  dim: 0x1a3a1a,
  grid: 0x2a5a2a,
  marker: 0x66ff66,
  glow: 0x22ff44,
} as const;

export const GLOBE_RADIUS = 1.5;
export const GRID_RADIUS = GLOBE_RADIUS * 1.0005;
export const COUNTRY_RADIUS = GLOBE_RADIUS * 1.001;
export const MARKER_RADIUS = GLOBE_RADIUS * 1.02;

// Shared materials
export const gridMaterial = new THREE.LineBasicMaterial({
  color: COLORS.grid,
  transparent: true,
  opacity: 0.4,
});

export const countryMaterial = new THREE.LineBasicMaterial({
  color: COLORS.primary,
  transparent: true,
  opacity: 0.8,
});

export const markerMaterial = new THREE.MeshBasicMaterial({
  color: COLORS.marker,
  transparent: true,
  opacity: 0.9,
  side: THREE.DoubleSide,
});
