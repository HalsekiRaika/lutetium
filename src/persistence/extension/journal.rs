use std::collections::BTreeMap;
use std::sync::Arc;

use crate::actor::{Context, FromContext};
use crate::persistence::{Batch, Event, SelectionCriteria};
use crate::persistence::errors::{DeserializeError, PersistError};
use crate::persistence::identifier::{PersistenceId, SequenceId};
use crate::system::ExtensionMissingError;

#[async_trait::async_trait]
pub trait JournalProvider: 'static + Sync + Send {
    async fn next(&self) -> Result<SequenceId, PersistError>;
    async fn insert(&self, id: &PersistenceId,seq: SequenceId, msg: Vec<u8>) -> Result<(), PersistError>;
    async fn delete(&self, seq: SequenceId) -> Result<(), PersistError>;
    async fn select_one(&self, id: &PersistenceId, seq: SequenceId) -> Result<Vec<u8>, PersistError>;
    async fn select_many(&self, id: &PersistenceId, criteria: SelectionCriteria) -> Result<BTreeMap<SequenceId, Vec<u8>>, PersistError>;
}

pub struct JournalProtocol(Arc<dyn JournalProvider>);

impl JournalProtocol {
    pub fn new(provider: impl JournalProvider) -> JournalProtocol {
        Self(Arc::new(provider))
    }
    
    pub async fn insert<E: Event>(&self, id: &PersistenceId, event: &E) -> Result<(), PersistError> {
        let next = self.0.next().await?;
        self.0.insert(id, next, event.as_bytes()?).await
    }
    
    pub async fn select_one<E: Event>(&self, id: &PersistenceId, seq: SequenceId) -> Result<E, PersistError> {
        self.0.select_one(id, seq).await
            .map(|bin| E::from_bytes(&bin))?
            .map_err(Into::into)
        
    }
    
    pub async fn select_many<E: Event>(&self, id: &PersistenceId, criteria: SelectionCriteria) -> Result<Batch<E>, PersistError> {
        let batch = self.0.select_many(id, criteria).await?
            .into_iter()
            .map(|(id, bin)| Ok((id, E::from_bytes(&bin)?)))
            .collect::<Result<BTreeMap<SequenceId, E>, DeserializeError>>()?;
        Ok(Batch::new(batch))
    }
}

impl Clone for JournalProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait::async_trait]
impl FromContext for JournalProtocol {
    type Rejection = ExtensionMissingError;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        ctx.system()
            .ext
            .get::<JournalProtocol>()
            .ok_or(ExtensionMissingError {
                module: "JournalProtocol"
            })
            .cloned()
    }
}
