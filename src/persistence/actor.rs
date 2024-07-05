use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::persistence::identifier::PersistenceId;

#[async_trait::async_trait]
pub trait PersistenceActor: 'static + Sync + Send
    where Self: DeserializeOwned
              + Serialize
{
    fn persistence_id(&self) -> PersistenceId;
}

