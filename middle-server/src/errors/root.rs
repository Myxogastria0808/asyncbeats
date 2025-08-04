#[derive(Debug, thiserror::Error)]
pub enum RootError {
    #[error(transparent)]
    SetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
