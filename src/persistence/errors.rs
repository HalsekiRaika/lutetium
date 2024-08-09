use crate::persistence::identifier::PersistenceId;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("PersistenceModule does not exist in extension in ActorSystem.")]
    Missing,
    
    #[error("")]
    Provider,
    
    #[error("{id}")]
    NotFound {
        id: PersistenceId
    },
    
    #[error("")]
    Selection,
    
    #[error(transparent)]
    Serialization(Box<dyn std::error::Error + Sync + Send>)
}

pub struct SerializeError(Box<dyn std::error::Error + Sync + Send>);
pub struct DeserializeError(Box<dyn std::error::Error + Sync + Send>);


impl<E: serde::ser::Error + Sync + Send + 'static> From<E> for SerializeError {
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}

impl<E: serde::de::Error + Sync + Send + 'static> From<E> for DeserializeError {
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}

impl From<SerializeError> for PersistError {
    fn from(value: SerializeError) -> Self {
        Self::Serialization(value.0)
    }
}

impl From<DeserializeError> for PersistError {
    fn from(value: DeserializeError) -> Self {
        Self::Serialization(value.0)
    }
}