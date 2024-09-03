use std::collections::BTreeMap;
use std::sync::Arc;

use crate::actor::{Context, FromContext};
use crate::persistence::{Batch, Event};
use crate::persistence::errors::{DeserializeError, PersistError};
use crate::persistence::identifier::SequenceId;
use crate::system::ExtensionMissingError;

#[async_trait::async_trait]
pub trait JournalProvider: 'static + Sync + Send {
    async fn next(&self) -> Result<SequenceId, PersistError>;
    async fn insert(&self, seq: SequenceId, msg: Vec<u8>) -> Result<(), PersistError>;
    async fn delete(&self, seq: SequenceId) -> Result<(), PersistError>;
    async fn select_one(&self, seq: SequenceId) -> Result<Vec<u8>, PersistError>;
    async fn select_many(&self, seq: SequenceId) -> Result<BTreeMap<SequenceId, Vec<u8>>, PersistError>;
}

pub struct JournalProtocol(Arc<dyn JournalProvider>);

impl JournalProtocol {
    pub fn new(provider: impl JournalProvider) -> JournalProtocol {
        Self(Arc::new(provider))
    }
    
    pub async fn insert<E: Event>(&self, event: &E) -> Result<(), PersistError> {
        let next = self.0.next().await?;
        self.0.insert(next, event.as_bytes()?).await
    }
    
    pub async fn select_one<E: Event>(&self, id: SequenceId) -> Result<E, PersistError> {
        self.0.select_one(id).await
            .map(|bin| E::from_bytes(&bin))?
            .map_err(Into::into)
        
    }
    
    pub async fn select_many<E: Event>(&self, criteria: SequenceId) -> Result<Batch<E>, PersistError> {
        let batch = self.0.select_many(criteria).await?
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

pub struct SelectionCriteria {
    min: SequenceId,
    max: SequenceId,
}

impl SelectionCriteria {
    pub const LATEST: SelectionCriteria = SelectionCriteria { min: SequenceId::MIN, max: SequenceId::MAX };
    
    pub fn new(min: SequenceId, max: SequenceId) -> Result<SelectionCriteria, PersistError> {
        if min > max || min >= max { 
            return Err(PersistError::Selection)
        }
        Ok(Self { min, max })
    }
    
    pub fn matches(&self, seq: &SequenceId) -> bool {
        &self.min <= seq && seq <= &self.max 
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
