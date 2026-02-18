use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;

/// Thread-safe audio capture manager.
/// The cpal::Stream lives on a dedicated thread; only the shared buffer crosses threads.
pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    stop_signal: Mutex<Option<std::sync::mpsc::Sender<()>>>,
    ready_signal: Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(0)),
            stop_signal: Mutex::new(None),
            ready_signal: Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new())),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        // If already recording, stop first
        if self.stop_signal.lock().unwrap().is_some() {
            log::warn!("Already recording, stopping previous recording");
            self.stop_inner();
        }

        self.buffer.lock().unwrap().clear();

        let buffer = self.buffer.clone();
        let sample_rate_store = self.sample_rate.clone();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let ready = self.ready_signal.clone();

        // Reset ready signal
        *ready.0.lock().unwrap() = false;
        *self.stop_signal.lock().unwrap() = Some(stop_tx);

        thread::spawn(move || {
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    log::error!("No input device available");
                    return;
                }
            };

            log::info!("Using input device: {:?}", device.name());

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to get input config: {e}");
                    return;
                }
            };

            let rate = config.sample_rate().0;
            let channels = config.channels() as usize;
            *sample_rate_store.lock().unwrap() = rate;

            log::info!(
                "Input config: {} ch, {} Hz, {:?}",
                channels,
                rate,
                config.sample_format()
            );

            let err_fn = |err: cpal::StreamError| {
                log::error!("Audio stream error: {}", err);
            };

            let buf = buffer.clone();
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut b = buf.lock().unwrap();
                        for chunk in data.chunks(channels) {
                            if let Some(&sample) = chunk.first() {
                                b.push(sample);
                            }
                        }
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => {
                    let buf16 = buffer;
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let mut b = buf16.lock().unwrap();
                            for chunk in data.chunks(channels) {
                                if let Some(&sample) = chunk.first() {
                                    b.push(sample as f32 / i16::MAX as f32);
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                format => {
                    log::error!("Unsupported sample format: {:?}", format);
                    return;
                }
            };

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to build input stream: {e}");
                    return;
                }
            };

            if let Err(e) = stream.play() {
                log::error!("Failed to start stream: {e}");
                return;
            }

            log::info!("Audio recording started — stream is live");

            // Signal that recording is ready
            {
                let (lock, cvar) = &*ready;
                let mut started = lock.lock().unwrap();
                *started = true;
                cvar.notify_all();
            }

            // Block until stop signal
            let _ = stop_rx.recv();
            drop(stream);
            log::info!("Audio recording thread stopped");
        });

        // Wait for the stream to actually start (up to 2s)
        let (lock, cvar) = &*self.ready_signal;
        let started = lock.lock().unwrap();
        let result = cvar
            .wait_timeout(started, std::time::Duration::from_secs(2))
            .unwrap();
        if !*result.0 {
            log::warn!("Audio stream took >2s to start");
        } else {
            log::info!("Audio stream confirmed ready");
        }

        Ok(())
    }

    fn stop_inner(&self) {
        if let Some(tx) = self.stop_signal.lock().unwrap().take() {
            let _ = tx.send(());
        }
    }

    /// Stop recording and return the captured audio as 16kHz mono f32 samples.
    pub fn stop(&self) -> Result<Vec<f32>, String> {
        self.stop_inner();

        // Brief pause to let the recording thread finish
        thread::sleep(std::time::Duration::from_millis(50));

        let buffer = self.buffer.lock().unwrap().clone();
        let sample_rate = *self.sample_rate.lock().unwrap();

        // Compute audio level metrics
        let rms = if buffer.is_empty() {
            0.0
        } else {
            (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt()
        };
        let peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        log::info!(
            "Audio captured: {} samples at {} Hz ({:.1}s), RMS={:.4}, peak={:.4}",
            buffer.len(),
            sample_rate,
            if sample_rate > 0 {
                buffer.len() as f32 / sample_rate as f32
            } else {
                0.0
            },
            rms,
            peak,
        );

        if buffer.is_empty() || sample_rate == 0 {
            return Ok(Vec::new());
        }

        let resampled = if sample_rate == 16000 {
            buffer
        } else {
            downsample(&buffer, sample_rate, 16000)
        };

        log::info!("Resampled to {} samples at 16kHz", resampled.len());
        Ok(resampled)
    }
}

fn downsample(buffer: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (buffer.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(new_len);
    for i in 0..new_len {
        let src_idx = i as f64 * ratio;
        let low = src_idx as usize;
        let high = (low + 1).min(buffer.len() - 1);
        let frac = (src_idx - low as f64) as f32;
        result.push(buffer[low] * (1.0 - frac) + buffer[high] * frac);
    }
    result
}
