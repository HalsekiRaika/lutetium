use crate::persistence::identifier::PersistenceId;
use crate::system::ExtensionMissingError;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("{id}")]
    NotFound {
        id: PersistenceId
    },
    
    #[error(transparent)]
    MissingExtension(ExtensionMissingError),

    #[error("")]
    Selection,

    #[error(transparent)]
    Serialization(Box<dyn std::error::Error + Sync + Send>)
}

impl From<ExtensionMissingError> for PersistError {
    fn from(value: ExtensionMissingError) -> Self {
        Self::MissingExtension(value)
    }
}