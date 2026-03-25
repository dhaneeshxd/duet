use crate::db::queries::Track;
use crate::errors::{AppError, AppResult};
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AudioInfo {
    pub sample_rate: u32,
    pub bits_per_sample: u32,
    pub channels: u8,
    pub bitrate_kbps: u32,
    pub codec: String,
    pub is_lossless: bool,
}

pub struct RichMetadata {
    pub track: Track,
    pub album_art: Option<Vec<u8>>,
    pub lyrics: Option<String>,
    pub audio_info: Option<AudioInfo>,
}

pub fn extract_metadata(path: &str) -> AppResult<Track> {
    extract_rich_metadata(path).map(|r| r.track)
}

pub fn extract_rich_metadata(path: &str) -> AppResult<RichMetadata> {
    let file = std::fs::File::open(path).map_err(|e| AppError::Io(e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| AppError::Audio(format!("Cannot read audio file: {}", path)))?;

    let mut format = probed.format;

    let mut title = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();
    let mut artist = "Unknown Artist".to_string();
    let mut album = "Unknown Album".to_string();
    let mut lyrics: Option<String> = None;
    let mut album_art: Option<Vec<u8>> = None;

    if let Some(metadata) = format.metadata().current() {
        for tag in metadata.tags() {
            match tag.std_key {
                Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                    title = tag.value.to_string();
                }
                Some(symphonia::core::meta::StandardTagKey::Artist) => {
                    artist = tag.value.to_string();
                }
                Some(symphonia::core::meta::StandardTagKey::Album) => {
                    album = tag.value.to_string();
                }
                Some(symphonia::core::meta::StandardTagKey::Lyrics) => {
                    lyrics = Some(tag.value.to_string());
                }
                _ => {}
            }
        }
        for visual in metadata.visuals() {
            album_art = Some(visual.data.to_vec());
            break;
        }
    }

    // Extract technical info from first audio track
    let audio_info = if let Some(track) = format.tracks().first() {
        let codec_params = &track.codec_params;

        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params.channels.map(|c| c.count() as u8).unwrap_or(2);
        let bits_per_sample = codec_params
            .bits_per_sample
            .or(codec_params.bits_per_coded_sample)
            .unwrap_or(16);

        // Duration in seconds
        let duration_secs = if let Some(n_frames) = codec_params.n_frames {
            if sample_rate > 0 {
                n_frames / sample_rate as u64
            } else {
                0
            }
        } else {
            0
        };

        // Estimate bitrate from file size and duration
        let bitrate_kbps = {
            let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            if duration_secs > 0 && file_size > 0 {
                ((file_size * 8) / duration_secs / 1000) as u32
            } else {
                0
            }
        };

        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_uppercase();

        let is_lossless = matches!(
            ext.as_str(),
            "FLAC" | "WAV" | "AIFF" | "ALAC" | "APE" | "WV"
        );

        let codec = match ext.as_str() {
            "FLAC" => "FLAC".to_string(),
            "WAV" => "PCM".to_string(),
            "MP3" => "MP3".to_string(),
            "AAC" | "M4A" => "AAC".to_string(),
            "OGG" => "Vorbis".to_string(),
            "OPUS" => "Opus".to_string(),
            "WMA" => "WMA".to_string(),
            other => other.to_string(),
        };

        Some(AudioInfo {
            sample_rate,
            bits_per_sample,
            channels,
            bitrate_kbps,
            codec,
            is_lossless,
        })
    } else {
        None
    };

    // Duration
    let duration_secs = if let Some(track) = format.tracks().first() {
        if let Some(n_frames) = track.codec_params.n_frames {
            if let Some(sr) = track.codec_params.sample_rate {
                n_frames / sr as u64
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    let track = Track {
        id: 0,
        path: path.to_string(),
        title,
        artist,
        album,
        duration_secs: duration_secs as i64,
    };

    Ok(RichMetadata {
        track,
        album_art,
        lyrics,
        audio_info,
    })
}
