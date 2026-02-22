import * as THREE from 'three';
import { MARKER_RADIUS } from './materials';

const DEG2RAD = Math.PI / 180;

export interface EventMarker {
  id: string;
  lat: number;
  lon: number;
  label: string;
  headline: string;
  source: string;
  severity: 'low' | 'medium' | 'high';
  category: 'weather' | 'news';
}

// Blue shades for weather, red shades for news — severity controls brightness
const MARKER_COLORS: Record<EventMarker['category'], Record<EventMarker['severity'], number>> = {
  weather: { low: 0x1a4a8a, medium: 0x2a7fff, high: 0x80c8ff },
  news:    { low: 0x8a1a1a, medium: 0xff3a3a, high: 0xff9090 },
};


function latLonToPosition(lat: number, lon: number, r: number): THREE.Vector3 {
  const phi = (90 - lat) * DEG2RAD;
  const theta = (lon + 180) * DEG2RAD;
  return new THREE.Vector3(
    -r * Math.sin(phi) * Math.cos(theta),
    r * Math.cos(phi),
    r * Math.sin(phi) * Math.sin(theta),
  );
}

export function createTriangleMarker(marker: EventMarker): THREE.Mesh {
  const size = marker.severity === 'high' ? 0.04 : marker.severity === 'medium' ? 0.03 : 0.025;
  const geometry = new THREE.ConeGeometry(size, size * 2, 3);
  const material = new THREE.MeshBasicMaterial({
    color: MARKER_COLORS[marker.category][marker.severity],
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

  mesh.userData = { label: marker.label, headline: marker.headline, source: marker.source, severity: marker.severity, category: marker.category };
  return mesh;
}

export function createMarkers(markers: EventMarker[] = []): THREE.Group {
  const group = new THREE.Group();
  for (const marker of markers) {
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
