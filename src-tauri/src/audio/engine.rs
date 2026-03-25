use crate::errors::{AppError, AppResult};
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct AudioEngine {
    sink: Arc<Sink>,
    volume: f32,
    pub current_path: Option<String>,
    stop_flag: Arc<AtomicBool>,
    playback_start: Option<(Instant, f64)>,
    pub manual_device: Option<String>,
    pub preferred_device: Option<String>,
    //current_device: Option<String>,
}

pub fn spawn_sink_pub() -> AppResult<Arc<Sink>> {
    let (tx, rx) = std::sync::mpsc::channel::<Arc<Sink>>();
    std::thread::spawn(move || match OutputStream::try_default() {
        Ok((_stream, handle)) => match Sink::try_new(&handle) {
            Ok(sink) => {
                let sink = Arc::new(sink);
                let _ = tx.send(Arc::clone(&sink));
                loop {
                    std::thread::sleep(Duration::from_secs(3600));
                }
            }
            Err(e) => eprintln!("Sink error: {}", e),
        },
        Err(e) => eprintln!("Stream error: {}", e),
    });
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| AppError::Audio("Audio thread timed out".to_string()))
}

pub fn spawn_sink_for_device(device_name: &str) -> AppResult<Arc<Sink>> {
    let host = rodio::cpal::default_host();
    let device = host
        .output_devices()
        .map_err(|e| AppError::Audio(e.to_string()))?
        .find(|d: &rodio::cpal::Device| d.name().ok().as_deref() == Some(device_name))
        .ok_or_else(|| AppError::Audio("Device not found".to_string()))?;

    let (tx, rx) = std::sync::mpsc::channel::<AppResult<Arc<Sink>>>();
    std::thread::spawn(move || match OutputStream::try_from_device(&device) {
        Ok((_stream, handle)) => match Sink::try_new(&handle) {
            Ok(sink) => {
                let _ = tx.send(Ok(Arc::new(sink)));
                loop {
                    std::thread::sleep(Duration::from_secs(3600));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(AppError::Audio(e.to_string())));
            }
        },
        Err(e) => {
            let _ = tx.send(Err(AppError::Audio(e.to_string())));
        }
    });
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| AppError::Audio("Device sink timed out".to_string()))?
}

impl AudioEngine {
    pub fn new() -> AppResult<Self> {
        let sink = spawn_sink_pub()?;
        Ok(AudioEngine {
            sink,
            volume: 1.0,
            current_path: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            playback_start: None,
            manual_device: None,
            preferred_device: None,
            // current_device: None,
        })
    }

    pub fn position(&self) -> f64 {
        match &self.playback_start {
            Some((t, offset)) => offset + t.elapsed().as_secs_f64(),
            None => 0.0,
        }
    }

    pub fn set_sink(&mut self, sink: Arc<Sink>) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();
        self.sink = sink;
        self.stop_flag = Arc::new(AtomicBool::new(false));
    }

    pub fn stop_sink(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();
    }
    pub fn spawn_sink_current(&self) -> AppResult<Arc<Sink>> {
        match &self.manual_device {
            Some(name) => spawn_sink_for_device(name),
            None => spawn_sink_pub(),
        }
    }
    pub fn play(&mut self, path: &str) -> AppResult<()> {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();

        let new_sink = self.spawn_sink_current()?;
        new_sink.set_volume(self.volume);
        let file = File::open(path).map_err(|e| AppError::Io(e))?;
        let source =
            Decoder::new(BufReader::new(file)).map_err(|e| AppError::Audio(e.to_string()))?;

        new_sink.append(source);
        self.sink = new_sink;
        self.current_path = Some(path.to_string());
        self.stop_flag = Arc::new(AtomicBool::new(false));
        self.playback_start = Some((Instant::now(), 0.0));

        Ok(())
    }

    pub fn play_from(&mut self, path: &str, secs: f64) -> AppResult<()> {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();

        let new_stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&new_stop_flag);
        let new_sink = self.spawn_sink_current()?;
        new_sink.set_volume(self.volume);
        let sink_arc = Arc::clone(&new_sink);
        self.sink = new_sink;
        self.current_path = Some(path.to_string());
        self.stop_flag = new_stop_flag;
        self.playback_start = Some((Instant::now(), secs));
        let path_clone = path.to_string();

        std::thread::spawn(move || {
            use rodio::buffer::SamplesBuffer;
            use symphonia::core::audio::SampleBuffer;
            use symphonia::core::codecs::DecoderOptions;
            use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
            use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
            use symphonia::core::meta::MetadataOptions;
            use symphonia::core::probe::Hint;
            use symphonia::core::units::Time;

            let file = match std::fs::File::open(&path_clone) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Open error: {}", e);
                    return;
                }
            };

            let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());
            let mut hint = Hint::new();
            if let Some(ext) = std::path::Path::new(&path_clone)
                .extension()
                .and_then(|e| e.to_str())
            {
                hint.with_extension(ext);
            }

            let probed = match symphonia::default::get_probe().format(
                &hint,
                mss,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            ) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Probe error: {}", e);
                    return;
                }
            };

            let mut format = probed.format;
            let track = match format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            {
                Some(t) => t.clone(),
                None => {
                    eprintln!("No audio track");
                    return;
                }
            };

            let track_id = track.id;
            let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
            let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2) as u16;

            if secs > 0.0 {
                if let Err(e) = format.seek(
                    SeekMode::Accurate,
                    SeekTo::Time {
                        time: Time::from(secs),
                        track_id: Some(track_id),
                    },
                ) {
                    eprintln!("Seek error: {}", e);
                }
            }

            let mut decoder = match symphonia::default::get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())
            {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Decoder error: {}", e);
                    return;
                }
            };

            loop {
                if stop_flag_clone.load(Ordering::SeqCst) {
                    break;
                }

                let packet = match format.next_packet() {
                    Ok(p) => p,
                    Err(_) => break,
                };
                if packet.track_id() != track_id {
                    continue;
                }

                let decoded = match decoder.decode(&packet) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
                sample_buf.copy_interleaved_ref(decoded);
                let samples = sample_buf.samples().to_vec();
                sink_arc.append(SamplesBuffer::new(channels, sample_rate, samples));

                while sink_arc.len() > 5 {
                    if stop_flag_clone.load(Ordering::SeqCst) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        });

        Ok(())
    }
    pub fn start_device_watcher(engine: Arc<Mutex<AudioEngine>>) {
        std::thread::spawn(move || {
            let mut last_available_count: usize = {
                let host = rodio::cpal::default_host();
                host.output_devices().map(|devs| devs.count()).unwrap_or(0)
            };
            let mut last_device_name: String = {
                let host = rodio::cpal::default_host();
                host.default_output_device()
                    .and_then(|d: rodio::cpal::Device| d.name().ok())
                    .unwrap_or_default()
            };

            loop {
                std::thread::sleep(Duration::from_millis(500));

                let host = rodio::cpal::default_host();

                let available_devices: Vec<String> = host
                    .output_devices()
                    .map(|devs| {
                        devs.filter_map(|d: rodio::cpal::Device| d.name().ok())
                            .collect()
                    })
                    .unwrap_or_default();

                let current_default: String = host
                    .default_output_device()
                    .and_then(|d: rodio::cpal::Device| d.name().ok())
                    .unwrap_or_default();

                let current_count = available_devices.len();

                if let Ok(mut eng) = engine.lock() {
                    // Any unplug event clear manual override entirely
                    if current_count < last_available_count {
                        eng.manual_device = None;
                        eng.preferred_device = None;
                    }

                    // Skip auto-switch if manual override still active
                    if eng.manual_device.is_some() {
                        last_available_count = current_count;
                        last_device_name = current_default;
                        continue;
                    }

                    // Auto-switch on default device change
                    if current_default != last_device_name {
                        last_device_name = current_default.clone();

                        let was_playing = eng.is_playing();
                        let path = eng.current_path.clone();
                        let pos = eng.position();

                        if let Ok(new_sink) = spawn_sink_pub() {
                            new_sink.set_volume(eng.volume);
                            eng.set_sink(new_sink);
                        }

                        if was_playing {
                            if let Some(ref p) = path {
                                let _ = eng.play_from(p, pos);
                            }
                        }
                    }
                }

                last_available_count = current_count;
            }
        });
    }

    pub fn pause(&self) {
        self.sink.pause();
    }
    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.sink.stop();
        self.current_path = None;
        self.playback_start = None;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }
    pub fn volume(&self) -> f32 {
        self.volume
    }
}

unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}
