import { invoke } from '@tauri-apps/api/core';
import type { EventMarker } from '../globe/markers';

export async function getEvents(): Promise<EventMarker[]> {
  return invoke<EventMarker[]>('get_events');
}
