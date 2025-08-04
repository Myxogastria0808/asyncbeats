pub struct AudioInfo {
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

impl From<hound::WavSpec> for AudioInfo {
    fn from(spec: hound::WavSpec) -> Self {
        let pcm_format = match spec.sample_format {
            hound::SampleFormat::Float => "float".to_string(),
            hound::SampleFormat::Int => "int".to_string(),
        };

        AudioInfo {
            channels: spec.channels,
            sample_rate: spec.sample_rate,
            bits_per_sample: spec.bits_per_sample,
            pcm_format,
        }
    }
}
