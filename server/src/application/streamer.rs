use crate::errors::streamer::StreamerError;
use axum::extract::ws::WebSocket;

pub async fn wave_streamer(socket: &mut WebSocket) -> Result<(), StreamerError> {
    // read wav file
    let mut reader = hound::WavReader::open("data/sample3.wav")?;
    // get headers
    let spec = reader.spec();
    tracing::info!(
        "WAV: {}Hz, {}ch, {}bits, {:?}",
        spec.sample_rate,
        spec.channels,
        spec.bits_per_sample,
        spec.sample_format
    );
    // get body (PCM samples)
    let mut samples = reader.samples::<i16>();
    let frames_per_chunk = 1024;
    let samples_per_chunk = frames_per_chunk * spec.channels as usize;
    // define interval
    let interval =
        tokio::time::Duration::from_secs_f64(frames_per_chunk as f64 / spec.sample_rate as f64);

    // send PCM data to middle-server
    loop {
        let mut buf = Vec::with_capacity(samples_per_chunk);

        // send chunks of a sample
        for _ in 0..samples_per_chunk {
            if let Some(Ok(sample)) = samples.next() {
                buf.extend_from_slice(&sample.to_le_bytes());
            }
        }

        // break point
        if buf.is_empty() {
            break;
        }

        // send PCM data
        /*
            binary size = frames_per_chunk × spec.channels × (spec.bits_per_sample / 8)
            NOTE: (spec.bits_per_sample / 8) -> bit size to byte size conversion
            e.g. 1024 frames × 2 channels × (16 bits / 8) = 4096 bytes
            e.g. 1024 frames × 1 channel × (16 bits / 8) = 2048 bytes
        */
        socket
            .send(axum::extract::ws::Message::Binary(buf.into()))
            .await
            .map_err(StreamerError::AxumError)?;

        tokio::time::sleep(interval).await;
    }

    Ok(())
}
