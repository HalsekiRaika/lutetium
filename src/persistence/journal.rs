use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::persistence::context::PersistContext;
use crate::persistence::errors::{DeserializeError, SerializeError};

pub trait Event: 'static + Sync + Send + Sized
    where Self: Serialize + DeserializeOwned
{
    const REGISTRY_KEY: &'static str;
    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializeError>;
}

#[async_trait::async_trait]
pub trait RecoverJournal<E>: 'static + Sync + Send + Sized
    where E: Event
{
    async fn recover_journal(this: &mut Option<Self>, event: E, ctx: &mut PersistContext);
}
