use std::sync::Arc;
use tokio::sync::RwLock;

pub type RwLockSharedState = Arc<RwLock<i32>>;
