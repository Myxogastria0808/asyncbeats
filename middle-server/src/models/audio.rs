use crate::errors::handler::HandlerError;
use tungstenite::Utf8Bytes;

pub type RwLockAudioInfo = std::sync::Arc<tokio::sync::RwLock<AudioInfo>>;

#[derive(Default, Debug, Clone)]
pub struct AudioInfo {
    /// The number of channels.
    pub channels: Option<u16>,

    /// The number of samples per second.
    ///
    /// A common value is 44100, this is 44.1 kHz which is used for CD audio.
    pub sample_rate: Option<u32>,

    /// The number of bits per sample.
    ///
    /// A common value is 16 bits per sample, which is used for CD audio.
    pub bits_per_sample: Option<u16>,

    /// Whether the wav's samples are float or integer values.
    pub pcm_format: Option<String>,
}

pub struct UnwrappedAudioInfo {
    /// The number of channels.
    pub channels: u16,

    /// The number of samples per second.
    ///
    /// A common value is 44100, this is 44.1 kHz which is used for CD audio.
    pub sample_rate: u32,

    /// The number of bits per sample.
    ///
    /// A common value is 16 bits per sample, which is used for CD audio.
    pub bits_per_sample: u16,

    /// Whether the wav's samples are float or integer values.
    pub pcm_format: String,
}

impl AudioInfo {
    pub fn get_audio_info(&self) -> Result<UnwrappedAudioInfo, Box<HandlerError>> {
        // get audio info from environment variable
        if let (Some(channels), Some(sample_rate), Some(bits_per_sample), Some(pcm_format)) = (
            self.channels,
            self.sample_rate,
            self.bits_per_sample,
            self.pcm_format.clone(),
        ) {
            Ok(UnwrappedAudioInfo {
                channels,
                sample_rate,
                bits_per_sample,
                pcm_format,
            })
        } else {
            Err(Box::new(HandlerError::AudioInfoUndefinedError))
        }
    }
}

impl TryFrom<Utf8Bytes> for AudioInfo {
    type Error = Box<HandlerError>;

    fn try_from(text: Utf8Bytes) -> Result<Self, Self::Error> {
        // transpile the Utf8Bytes to String
        let text = text.to_string();

        // validate the format elements
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() != 4 {
            return Err(Box::new(HandlerError::ParseAudioInfoError(text)));
        }

        // get channels
        let channels: u16 = parts[0]
            .parse()
            .map_err(|e| Box::new(HandlerError::ParseIntError(e)))?;
        // get sample_rate
        let sample_rate: u32 = parts[1]
            .parse()
            .map_err(|e| Box::new(HandlerError::ParseIntError(e)))?;
        // get bits_per_sample
        let bits_per_sample: u16 = parts[2]
            .parse()
            .map_err(|e| Box::new(HandlerError::ParseIntError(e)))?;
        // get pcm_format
        let pcm_format = match parts[3] {
            "float" => "float".to_string(),
            "int" => "int".to_string(),
            _ => Err(Box::new(HandlerError::ParseAudioInfoError(format!(
                "Invalid PCM format: {}",
                parts[3]
            ))))?,
        };

        Ok(AudioInfo {
            channels: Some(channels),
            sample_rate: Some(sample_rate),
            bits_per_sample: Some(bits_per_sample),
            pcm_format: Some(pcm_format),
        })
    }
}
