pub type RwLockDelayFlag = std::sync::Arc<tokio::sync::RwLock<DelayFlag>>;

pub enum DelayFlag {
    // The delay is selected, when the middle-server start sending PCM data.
    Initialized,
    // The delay is selected, when the middle server sends PCM data a certain number of times.
    // The certain number sets in the environment variable `DELAY_THRESHOLD`.
    Enabled,
    // The delay is selected, when the middle server sends PCM data more than the above number of times.
    Disabled,
}
