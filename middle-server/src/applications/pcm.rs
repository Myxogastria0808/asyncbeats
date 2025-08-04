use crate::{errors::handler::HandlerError, models::packet::WindowPacket};
use std::collections::VecDeque;

// [task3] pcm data processing
pub async fn pcm_data_processing(
    window_size: u64,
    slide_size: u64,
    mut pcm_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    window_tx: tokio::sync::mpsc::Sender<WindowPacket>,
) -> Result<(), HandlerError> {
    let mut counter: u64 = 0;
    let mut stock_buffer: VecDeque<Vec<u8>> = VecDeque::new();
    let mut window_packet: Vec<u8> = Vec::new();

    //* step6: receive binary from sender (producer) *//
    //* step7: do sliding window (while loop) *//
    //? Receiver (Consumer) //
    while let Some(bin) = pcm_rx.recv().await {
        //* collect buffer *//
        stock_buffer.push_back(bin);
        counter += 1;

        tracing::info!("counter: {}", counter);

        if counter >= window_size {
            // create window packet
            for _ in 0..slide_size {
                if let Some(buf) = stock_buffer.pop_front() {
                    window_packet.extend(buf);
                }
            }

            //* step8: send window packet to window_data_processing with window size *//
            //? Sender (Producer) //
            window_tx.send(WindowPacket(window_packet.clone())).await?;

            // reset counter
            counter -= slide_size;
            // reset send buffer
            window_packet.clear();
        }
    }
    Ok(())
}
