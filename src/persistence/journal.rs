use std::collections::BTreeMap;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::Context;
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::SequenceId;

pub trait Event: 'static + Sync + Send + Sized
    where Self: Serialize + DeserializeOwned
{ 
    fn as_bytes(&self) -> Result<Vec<u8>, PersistError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, PersistError>;   
}

#[async_trait::async_trait]
pub trait RecoverJournal<E>: 'static + Sync + Send 
    where E: Event
{
    async fn recover_journal(&mut self, event: E, ctx: &mut Context);
}

#[async_trait::async_trait]
pub trait RecoverJournalBatch<E>: 'static + Sync + Send 
    where Self: RecoverJournal<E>,
             E: Event
{
    async fn recover_journal_batch(&mut self, event: Batch<E>, ctx: &mut Context);
}

#[async_trait::async_trait]
impl<T: RecoverJournal<E>, E: Event> RecoverJournalBatch<E> for T {
    async fn recover_journal_batch(&mut self, event: Batch<E>, ctx: &mut Context) {
        for event in event.0.into_values() {
            self.recover_journal(event, ctx).await
        }
    }
}



#[derive(Debug)]
pub struct Batch<E: Event>(BTreeMap<SequenceId, E>);

impl<E: Event> Batch<E> {
    
}