import * as THREE from 'three';
import { COLORS, GLOBE_RADIUS } from './materials';
import { createGridLines } from './wireframe';
import { createCountryOutlines } from './countries';
import { createMarkers, animateMarkers, createTriangleMarker, type EventMarker } from './markers';
import {
  createInteractionState,
  setupInteractions,
  updateInteractions,
  type HoverData,
} from './interactions';

export interface GlobeScene {
  destroy: () => void;
  updateMarkers: (events: EventMarker[]) => void;
}

export function initGlobeScene(container: HTMLElement, onHover?: (data: HoverData) => void): GlobeScene {
  // Renderer
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.setClearColor(COLORS.background);
  container.appendChild(renderer.domElement);

  // Scene
  const scene = new THREE.Scene();

  // Camera
  const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
  camera.position.set(0, 0, 4.5);

  // Globe group — all globe elements rotate together
  const globeGroup = new THREE.Group();
  scene.add(globeGroup);

  // Atmosphere glow — Fresnel rim glow (bright at edges, transparent at center)
  const glowGeometry = new THREE.SphereGeometry(GLOBE_RADIUS * 1.12, 64, 64);
  const glowMaterial = new THREE.ShaderMaterial({
    transparent: true,
    side: THREE.FrontSide,
    depthWrite: false,
    uniforms: {
      glowColor: { value: new THREE.Color(COLORS.glow) },
    },
    vertexShader: `
      varying vec3 vNormal;
      varying vec3 vViewDir;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vec4 mvPos = modelViewMatrix * vec4(position, 1.0);
        vViewDir = normalize(-mvPos.xyz);
        gl_Position = projectionMatrix * mvPos;
      }
    `,
    fragmentShader: `
      uniform vec3 glowColor;
      varying vec3 vNormal;
      varying vec3 vViewDir;
      void main() {
        float fresnel = 1.0 - dot(vNormal, vViewDir);
        float rim = pow(fresnel, 3.5) * 0.6;
        gl_FragColor = vec4(glowColor, rim);
      }
    `,
  });
  const glowMesh = new THREE.Mesh(glowGeometry, glowMaterial);
  globeGroup.add(glowMesh);

  // Globe wireframe sphere (faint outline)
  const sphereGeometry = new THREE.SphereGeometry(GLOBE_RADIUS, 48, 48);
  const sphereWireframe = new THREE.WireframeGeometry(sphereGeometry);
  const sphereLines = new THREE.LineSegments(
    sphereWireframe,
    new THREE.LineBasicMaterial({ color: COLORS.dim, transparent: true, opacity: 0.15 }),
  );
  globeGroup.add(sphereLines);

  // Lat/lon grid
  globeGroup.add(createGridLines());

  // Country outlines
  globeGroup.add(createCountryOutlines());

  // Event markers
  const markers = createMarkers();
  globeGroup.add(markers);

  // Interactions
  const interactionState = createInteractionState();
  const cleanupInteractions = setupInteractions(renderer.domElement, globeGroup, interactionState, camera, markers, onHover);

  // Resize handler
  function resize() {
    const width = container.clientWidth;
    const height = container.clientHeight;
    renderer.setSize(width, height);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
  }
  resize();
  window.addEventListener('resize', resize);

  // Animation loop
  let animationId: number;
  const clock = new THREE.Clock();

  function animate() {
    animationId = requestAnimationFrame(animate);
    const elapsed = clock.getElapsedTime();

    updateInteractions(globeGroup, interactionState);
    animateMarkers(markers, elapsed);

    renderer.render(scene, camera);
  }
  animate();

  return {
    destroy() {
      cancelAnimationFrame(animationId);
      window.removeEventListener('resize', resize);
      cleanupInteractions();
      renderer.dispose();
      renderer.domElement.remove();
    },
    updateMarkers(events: EventMarker[]) {
      for (const child of [...markers.children]) {
        markers.remove(child);
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose();
          (child.material as THREE.Material).dispose();
        }
      }
      for (const event of events) {
        markers.add(createTriangleMarker(event));
      }
    },
  };
}
