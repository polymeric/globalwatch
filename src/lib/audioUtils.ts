/**
 * Downsample Float32Array from sourceSampleRate to 16000 Hz using linear interpolation.
 */
export function downsampleTo16kHz(
  buffer: Float32Array,
  sourceSampleRate: number
): Float32Array {
  if (sourceSampleRate === 16000) return buffer;
  const ratio = sourceSampleRate / 16000;
  const newLength = Math.round(buffer.length / ratio);
  const result = new Float32Array(newLength);
  for (let i = 0; i < newLength; i++) {
    const srcIndex = i * ratio;
    const low = Math.floor(srcIndex);
    const high = Math.min(low + 1, buffer.length - 1);
    const frac = srcIndex - low;
    result[i] = buffer[low] * (1 - frac) + buffer[high] * frac;
  }
  return result;
}

/**
 * Convert Float32Array to little-endian byte array (Uint8Array).
 */
export function float32ToBytes(samples: Float32Array): Uint8Array {
  return new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength);
}

/** Preferred voices in order of priority (macOS names). */
const PREFERRED_VOICES = [
  'Daniel',      // British male, natural
  'Alex',        // US male, natural
  'Samantha',    // US female, natural
  'Karen',       // Australian female
  'Moira',       // Irish female
];

function pickVoice(): SpeechSynthesisVoice | null {
  const voices = window.speechSynthesis.getVoices();
  for (const name of PREFERRED_VOICES) {
    const match = voices.find((v) => v.name.includes(name) && v.lang.startsWith('en'));
    if (match) return match;
  }
  // Fallback: any English voice that isn't a compact/novelty voice
  return voices.find((v) => v.lang.startsWith('en') && !v.name.includes('Compact')) ?? null;
}

/**
 * Speak text using the browser's SpeechSynthesis API.
 * Returns a promise that resolves when speech ends.
 */
export function speakText(text: string): Promise<void> {
  return new Promise((resolve, reject) => {
    if (!window.speechSynthesis) {
      reject(new Error('SpeechSynthesis not supported'));
      return;
    }
    // Cancel any ongoing speech
    window.speechSynthesis.cancel();

    const utterance = new SpeechSynthesisUtterance(text);
    const voice = pickVoice();
    if (voice) {
      utterance.voice = voice;
      console.log('[tts] using voice:', voice.name);
    }
    utterance.rate = 1.0;
    utterance.pitch = 1.0;
    utterance.volume = 1.0;
    utterance.onend = () => resolve();
    utterance.onerror = (e) => reject(new Error(`TTS error: ${e.error}`));
    window.speechSynthesis.speak(utterance);
  });
}

/**
 * Play WAV bytes through Web Audio API. Returns a promise that resolves when playback ends.
 */
export async function playWavBytes(wavBytes: Uint8Array): Promise<void> {
  const audioCtx = new AudioContext();
  const arrayBuffer = wavBytes.buffer.slice(
    wavBytes.byteOffset,
    wavBytes.byteOffset + wavBytes.byteLength
  ) as ArrayBuffer;
  const audioBuffer = await audioCtx.decodeAudioData(arrayBuffer);
  const source = audioCtx.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(audioCtx.destination);
  return new Promise((resolve) => {
    source.onended = () => {
      audioCtx.close();
      resolve();
    };
    source.start();
  });
}
