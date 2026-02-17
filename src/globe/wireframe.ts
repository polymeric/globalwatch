import * as THREE from 'three';
import { GRID_RADIUS, gridMaterial } from './materials';

const DEG2RAD = Math.PI / 180;

/** Generate lat/lon grid lines on a sphere. */
export function createGridLines(): THREE.LineSegments {
  const positions: number[] = [];
  const segments = 128; // points per line for smooth curves

  // Meridians (longitude lines) — every 15 degrees
  for (let lon = -180; lon < 180; lon += 15) {
    for (let i = 0; i < segments; i++) {
      const lat1 = -90 + (180 * i) / segments;
      const lat2 = -90 + (180 * (i + 1)) / segments;
      pushLatLon(positions, lat1, lon, GRID_RADIUS);
      pushLatLon(positions, lat2, lon, GRID_RADIUS);
    }
  }

  // Parallels (latitude lines) — every 15 degrees
  for (let lat = -75; lat <= 75; lat += 15) {
    for (let i = 0; i < segments; i++) {
      const lon1 = -180 + (360 * i) / segments;
      const lon2 = -180 + (360 * (i + 1)) / segments;
      pushLatLon(positions, lat, lon1, GRID_RADIUS);
      pushLatLon(positions, lat, lon2, GRID_RADIUS);
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  return new THREE.LineSegments(geometry, gridMaterial);
}

function pushLatLon(arr: number[], lat: number, lon: number, r: number) {
  const phi = (90 - lat) * DEG2RAD;
  const theta = (lon + 180) * DEG2RAD;
  arr.push(
    -r * Math.sin(phi) * Math.cos(theta),
    r * Math.cos(phi),
    r * Math.sin(phi) * Math.sin(theta),
  );
}
