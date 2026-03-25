use cpal::traits::{DeviceTrait, HostTrait};

#[tauri::command]
pub fn list_audio_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices()
        .map(|devs| devs.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_current_audio_device() -> String {
    let host = cpal::default_host();
    host.default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[tauri::command]
pub async fn set_audio_device(
    state: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    device_name: String,
) -> Result<(), String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    use rodio::OutputStream;

    let host = cpal::default_host();
    let device = host
        .output_devices()
        .map_err(|e| e.to_string())?
        .find(|d| d.name().ok().as_deref() == Some(&device_name))
        .ok_or("Device not found")?;

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || match OutputStream::try_from_device(&device) {
        Ok((_stream, handle)) => match Sink::try_new(&handle) {
            Ok(sink) => {
                let _ = tx.send(Ok(Arc::new(sink)));
                loop {
                    std::thread::sleep(Duration::from_secs(3600));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e.to_string()));
            }
        },
        Err(e) => {
            let _ = tx.send(Err(e.to_string()));
        }
    });

    let new_sink = rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "Timeout".to_string())??;

    let mut eng = state.lock().map_err(|e| e.to_string())?;
    let was_playing = eng.is_playing();
    let path = eng.current_path.clone();
    let pos = eng.position();

    new_sink.set_volume(eng.volume);
    eng.set_sink(new_sink);

    if was_playing {
        if let Some(p) = path {
            eng.play_from(&p, pos).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
