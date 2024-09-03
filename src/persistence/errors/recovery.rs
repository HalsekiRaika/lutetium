use std::error::Error;
use crate::system::ExtensionMissingError;

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error(transparent)]
    MissingExtension(ExtensionMissingError),
    
    #[error(transparent)]
    Deserialization(Box<dyn Error + Sync + Send>)
}

impl From<ExtensionMissingError> for RecoveryError {
    fn from(value: ExtensionMissingError) -> Self {
        Self::MissingExtension(value)
    }
}