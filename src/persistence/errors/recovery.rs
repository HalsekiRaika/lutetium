use std::error::Error;
use crate::persistence::errors::PersistError;
use crate::system::ExtensionMissingError;

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error(transparent)]
    MissingExtension(ExtensionMissingError),
    
    #[error(transparent)]
    Deserialization(Box<dyn Error + Sync + Send>),
    
    #[error("A problem occurred at persist protocol.")]
    Persist(PersistError),
    
    #[error("data is not compatible with mapping.")]
    NotCompatible(&'static str)
}

impl From<ExtensionMissingError> for RecoveryError {
    fn from(value: ExtensionMissingError) -> Self {
        Self::MissingExtension(value)
    }
}

impl From<PersistError> for RecoveryError {
    fn from(value: PersistError) -> Self {
        Self::Persist(value)
    }
}