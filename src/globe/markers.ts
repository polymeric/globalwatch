import * as THREE from 'three';
import { MARKER_RADIUS, COLORS } from './materials';

const DEG2RAD = Math.PI / 180;

export interface EventMarker {
  lat: number;
  lon: number;
  label: string;
  severity: 'low' | 'medium' | 'high';
}

const SEVERITY_COLORS: Record<string, number> = {
  low: 0x2a8a2a,
  medium: COLORS.primary,
  high: COLORS.bright,
};

// Demo markers representing example events
const DEMO_MARKERS: EventMarker[] = [
  { lat: 35.6762, lon: 139.6503, label: 'Typhoon Warning — Tokyo', severity: 'high' },
  { lat: 25.276, lon: 55.2962, label: 'Sandstorm Alert — Dubai', severity: 'medium' },
  { lat: -33.8688, lon: 151.2093, label: 'Bushfire Risk — Sydney', severity: 'high' },
  { lat: 51.5074, lon: -0.1278, label: 'Flood Watch — London', severity: 'low' },
  { lat: 37.7749, lon: -122.4194, label: 'Earthquake Swarm — San Francisco', severity: 'medium' },
  { lat: -22.9068, lon: -43.1729, label: 'Landslide Warning — Rio de Janeiro', severity: 'high' },
];

function latLonToPosition(lat: number, lon: number, r: number): THREE.Vector3 {
  const phi = (90 - lat) * DEG2RAD;
  const theta = (lon + 180) * DEG2RAD;
  return new THREE.Vector3(
    -r * Math.sin(phi) * Math.cos(theta),
    r * Math.cos(phi),
    r * Math.sin(phi) * Math.sin(theta),
  );
}

function createTriangleMarker(marker: EventMarker): THREE.Mesh {
  const size = marker.severity === 'high' ? 0.04 : marker.severity === 'medium' ? 0.03 : 0.025;
  const geometry = new THREE.ConeGeometry(size, size * 2, 3);
  const material = new THREE.MeshBasicMaterial({
    color: SEVERITY_COLORS[marker.severity],
    transparent: true,
    opacity: 0.9,
    side: THREE.DoubleSide,
  });

  const mesh = new THREE.Mesh(geometry, material);
  const pos = latLonToPosition(marker.lat, marker.lon, MARKER_RADIUS);
  mesh.position.copy(pos);

  // Orient the cone to point outward from globe center
  mesh.lookAt(pos.clone().multiplyScalar(2));
  mesh.rotateX(Math.PI / 2);

  mesh.userData = { label: marker.label, severity: marker.severity };
  return mesh;
}

export function createMarkers(): THREE.Group {
  const group = new THREE.Group();
  for (const marker of DEMO_MARKERS) {
    group.add(createTriangleMarker(marker));
  }
  return group;
}

export function animateMarkers(group: THREE.Group, time: number) {
  for (const child of group.children) {
    if (child instanceof THREE.Mesh) {
      // Gentle pulse effect
      const scale = 1 + 0.15 * Math.sin(time * 3 + child.position.x * 10);
      child.scale.setScalar(scale);
    }
  }
}
