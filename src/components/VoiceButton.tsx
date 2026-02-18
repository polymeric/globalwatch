import { useState, useCallback, useEffect } from 'react';
import {
  checkModelsReady,
  getModelsDir,
  loadWhisperModel,
  startRecording,
  stopAndTranscribe,
  queryLlm,
  speakTextNative,
} from '../lib/tauriVoice';
import './VoiceButton.css';

export type VoicePipelineState =
  | 'idle'
  | 'recording'
  | 'processing'
  | 'speaking'
  | 'error';

export default function VoiceButton() {
  const [pipelineState, setPipelineState] =
    useState<VoicePipelineState>('idle');
  const [lastTranscript, setLastTranscript] = useState('');
  const [lastResponse, setLastResponse] = useState('');
  const [error, setError] = useState('');
  const [modelReady, setModelReady] = useState<boolean | null>(null);
  const [modelsDir, setModelsDir] = useState('');
  const [conversationHistory, setConversationHistory] = useState<
    Array<{ role: string; content: string }>
  >([]);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const ready = await checkModelsReady();
        if (cancelled) return;
        setModelReady(ready);
        if (ready) {
          await loadWhisperModel();
        } else {
          const dir = await getModelsDir();
          if (!cancelled) setModelsDir(dir);
        }
      } catch (err) {
        if (!cancelled) {
          setError(String(err));
          setModelReady(false);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const handlePointerDown = useCallback(async () => {
    if (!modelReady) return;
    setError('');
    setLastTranscript('');
    setLastResponse('');
    setPipelineState('recording');
    try {
      await startRecording();
      console.log('[voice] recording started (cpal)');
    } catch (err) {
      console.error('[voice] failed to start recording:', err);
      setError(String(err));
      setPipelineState('error');
      setTimeout(() => setPipelineState('idle'), 3000);
    }
  }, [modelReady]);

  const handlePointerUp = useCallback(async () => {
    if (pipelineState !== 'recording') return;
    setPipelineState('processing');
    try {
      console.log('[voice] stopping recording, transcribing...');
      const transcript = await stopAndTranscribe();
      console.log('[voice] transcript:', JSON.stringify(transcript));
      setLastTranscript(transcript);

      if (transcript) {
        console.log('[voice] querying LLM...');
        const recentHistory = conversationHistory.slice(-10);
        const response = await queryLlm(transcript, recentHistory);
        console.log('[voice] LLM response:', JSON.stringify(response));
        setLastResponse(response);
        setConversationHistory((prev) => [
          ...prev,
          { role: 'user', content: transcript },
          { role: 'assistant', content: response },
        ]);

        setPipelineState('speaking');
        await speakTextNative(response);
      } else {
        console.warn('[voice] empty transcript, skipping LLM');
      }

      setPipelineState('idle');
    } catch (err) {
      console.error('[voice] pipeline error:', err);
      setError(String(err));
      setPipelineState('error');
      setTimeout(() => setPipelineState('idle'), 3000);
    }
  }, [pipelineState, conversationHistory]);

  const stateLabel: Record<VoicePipelineState, string> = {
    idle: 'HOLD TO SPEAK',
    recording: 'RECORDING...',
    processing: 'PROCESSING...',
    speaking: 'SPEAKING...',
    error: 'ERROR',
  };

  if (modelReady === false) {
    return (
      <div className="voice-panel">
        <div className="voice-setup">
          WHISPER MODEL NOT FOUND
          <br />
          Place <strong>ggml-base.en.bin</strong> in:
          <code className="voice-setup__path">{modelsDir}</code>
        </div>
      </div>
    );
  }

  if (modelReady === null) {
    return (
      <div className="voice-panel">
        <div className="voice-setup">LOADING VOICE MODEL...</div>
      </div>
    );
  }

  return (
    <div className="voice-panel">
      <button
        className={`voice-btn voice-btn--${pipelineState}`}
        onPointerDown={handlePointerDown}
        onPointerUp={handlePointerUp}
        onPointerLeave={pipelineState === 'recording' ? handlePointerUp : undefined}
        disabled={
          pipelineState === 'processing' || pipelineState === 'speaking'
        }
      >
        <span className="voice-btn__icon">&#9654;</span>
        <span className="voice-btn__label">{stateLabel[pipelineState]}</span>
      </button>
      {lastTranscript && (
        <div className="voice-transcript">YOU: {lastTranscript}</div>
      )}
      {lastResponse && (
        <div className="voice-response">AI: {lastResponse}</div>
      )}
      {error && <div className="voice-error">{error}</div>}
    </div>
  );
}
