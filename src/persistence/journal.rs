use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::Context;
use crate::persistence::errors::PersistError;

pub trait Event: 'static + Sync + Send + Sized
    where Self: Serialize + DeserializeOwned
{ 
    fn as_bytes(&self) -> Result<Vec<u8>, PersistError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, PersistError>;   
}

pub trait RecoverJournal<E>: 'static + Sync + Send 
    where E: Event
{
    fn recover_journal(&mut self, event: E, ctx: &mut Context);
}