import { invoke } from '@tauri-apps/api/core';

export async function checkModelsReady(): Promise<boolean> {
  return invoke<boolean>('check_models_ready');
}

export async function getModelsDir(): Promise<string> {
  return invoke<string>('get_models_dir');
}

export async function loadWhisperModel(): Promise<void> {
  return invoke<void>('load_whisper_model');
}

export async function startRecording(): Promise<void> {
  return invoke<void>('start_recording');
}

export async function stopAndTranscribe(): Promise<string> {
  return invoke<string>('stop_and_transcribe');
}

export async function queryLlm(
  prompt: string,
  history: Array<{ role: string; content: string }>
): Promise<string> {
  return invoke<string>('query_llm', { prompt, history });
}

export async function speakTextNative(text: string): Promise<void> {
  return invoke<void>('speak_text', { text });
}
