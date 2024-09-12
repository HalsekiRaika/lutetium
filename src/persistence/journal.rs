use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::actor::{Context, FromContext};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::errors::{DeserializeError, RecoveryError, SerializeError};
use crate::persistence::identifier::{PersistenceId, SequenceId};
use crate::persistence::JournalProtocol;
use crate::persistence::recovery::FixtureParts;

pub trait Event: 'static + Sync + Send + Sized
    where Self: Serialize + DeserializeOwned
{ 
    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializeError>;
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
    pub fn new(batch: BTreeMap<SequenceId, E>) -> Batch<E> {
        Self(batch)
    }
}

impl<E: Event> Default for Batch<E> {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}


pub struct FixtureEvent<A: PersistenceActor>(Option<FixtureParts<A>>);

impl<A: PersistenceActor> FixtureEvent<A> {
    pub async fn from_id_with_context(id: &PersistenceId, ctx: &mut Context) -> Result<Self, RecoveryError> {
        let journal = JournalProtocol::from_context(ctx).await?;
        
        todo!()
    }
}