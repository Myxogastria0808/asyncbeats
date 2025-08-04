use crate::{
    errors::handler::HandlerError,
    models::{
        audio::{RwLockAudioInfo, UnwrappedAudioInfo},
        packet::{MessagePack, WindowPacket},
        ws::MutexWebSocketClientWriter,
    },
};
use axum::extract::ws::Message;
use futures_util::SinkExt;
use numpy::IntoPyArray;
use pyo3::{
    PyResult, Python,
    types::{PyAnyMethods, PyDict},
};

// [task4] window data processing
// TODO: ここで時間のかかる解析処理を実行する
pub async fn window_data_processing(
    mut window_rx: tokio::sync::mpsc::Receiver<WindowPacket>,
    shared_client_writer: MutexWebSocketClientWriter,
    shared_audio_info: RwLockAudioInfo,
) -> Result<(), HandlerError> {
    //? Receiver (Consumer) //
    while let Some(window_packet) = window_rx.recv().await {
        let binary = window_packet.0;

        //* step9: analyze pcm data *//
        let rwlock_audio_info = shared_audio_info.read().await;
        let audio_info = rwlock_audio_info.get_audio_info().map_err(|e| {
            tracing::error!("Failed to get audio info: {:?}", e);
            HandlerError::AudioInfoUndefinedError
        })?;
        drop(rwlock_audio_info); // release the lock

        // Convert binary data to f32 samples based on audio info
        let samples = binary_transformer(binary.clone(), &audio_info);
        let bpm = Python::with_gil(|py| pcm_detector(py, samples, audio_info.sample_rate as f64))?;

        //* step10: create message pack *//
        let message_pack = rmp_serde::to_vec_named(&MessagePack {
            pcm: binary.clone(),
            bpm,
        })?;

        //* step11: send messagepack to client *//
        let mut writer = shared_client_writer.lock().await;
        writer.send(Message::Binary(message_pack.into())).await?;
    }
    Ok(())
}

//TODO: 正しさの検証をする (正しくはない)
fn binary_transformer(binary: Vec<u8>, _audio_info: &UnwrappedAudioInfo) -> Vec<f32> {
    binary
        .chunks_exact(2) // 1. バイト列を2バイトずつのチャンクに分割
        .map(|chunk| {
            // 2. 2バイトのスライスをi16に変換 (リトルエンディアン)
            // try_into()で&[u8]を[u8; 2]に変換
            let val_i16 = i16::from_le_bytes(chunk.try_into().unwrap());
            // 3. i16の値をf32にキャストし、正規化
            val_i16 as f32 / 32768.0
        })
        .collect() // 4. 結果をVec<f32>に集める
}

fn pcm_detector<'py>(py: Python<'py>, samples: Vec<f32>, sample_rate: f64) -> PyResult<f64> {
    // [python code]
    // import librosa
    let librosa = py.import("librosa")?;

    // [python code]
    // kwargs = {"y": samples, "sr": sample_rate}
    let kwargs = PyDict::new(py);
    kwargs.set_item("y", samples.into_pyarray(py))?;
    kwargs.set_item("sr", sample_rate)?;

    // [python code]
    // tempo, _beats = librosa.beat.beat_track(kwargs)
    let (tempo, _beats) = librosa
        .getattr("beat")?
        .getattr("beat_track")?
        .call((), Some(&kwargs))?
        .extract::<(f64, pyo3::PyObject)>()?;

    Ok(tempo)
}
