use crate::persistence::errors::persist::PersistError;
use crate::persistence::errors::recovery::RecoveryError;

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

impl From<DeserializeError> for RecoveryError {
    fn from(value: DeserializeError) -> Self {
        Self::Deserialization(value.0)
    }
}