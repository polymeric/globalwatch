import * as THREE from 'three';
import { feature } from 'topojson-client';
import type { Topology } from 'topojson-specification';
import type { FeatureCollection, Geometry, Position } from 'geojson';
import { COUNTRY_RADIUS, countryMaterial } from './materials';
import topoData from '../data/countries-110m.json';

const DEG2RAD = Math.PI / 180;

function latLonToVec3(lat: number, lon: number, r: number): THREE.Vector3 {
  const phi = (90 - lat) * DEG2RAD;
  const theta = (lon + 180) * DEG2RAD;
  return new THREE.Vector3(
    -r * Math.sin(phi) * Math.cos(theta),
    r * Math.cos(phi),
    r * Math.sin(phi) * Math.sin(theta),
  );
}

function processRing(ring: Position[], positions: number[], r: number) {
  for (let i = 0; i < ring.length - 1; i++) {
    const [lon1, lat1] = ring[i];
    const [lon2, lat2] = ring[i + 1];
    // Skip segments that wrap around the antimeridian
    if (Math.abs(lon2 - lon1) > 170) continue;
    const v1 = latLonToVec3(lat1, lon1, r);
    const v2 = latLonToVec3(lat2, lon2, r);
    positions.push(v1.x, v1.y, v1.z, v2.x, v2.y, v2.z);
  }
}

function processGeometry(geom: Geometry, positions: number[], r: number) {
  switch (geom.type) {
    case 'Polygon':
      for (const ring of geom.coordinates) {
        processRing(ring, positions, r);
      }
      break;
    case 'MultiPolygon':
      for (const polygon of geom.coordinates) {
        for (const ring of polygon) {
          processRing(ring, positions, r);
        }
      }
      break;
  }
}

export function createCountryOutlines(): THREE.LineSegments {
  const topo = topoData as unknown as Topology;
  const objectKey = Object.keys(topo.objects)[0];
  const geojson = feature(topo, topo.objects[objectKey]) as FeatureCollection;

  const positions: number[] = [];

  for (const feat of geojson.features) {
    if (feat.geometry) {
      processGeometry(feat.geometry, positions, COUNTRY_RADIUS);
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  return new THREE.LineSegments(geometry, countryMaterial);
}
