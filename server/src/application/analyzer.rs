use crate::{errors::analyzer::AnalyzerError, models::audio::AudioInfo};

pub fn wave_analyzer() -> Result<AudioInfo, AnalyzerError> {
    // read wav file
    let reader = hound::WavReader::open("data/sample3.wav")?;

    // get headers
    let spec = reader.spec();
    tracing::info!(
        "WAV: {}Hz, {}ch, {}bits, {:?}",
        spec.sample_rate,
        spec.channels,
        spec.bits_per_sample,
        spec.sample_format
    );

    Ok(spec.into())
}
