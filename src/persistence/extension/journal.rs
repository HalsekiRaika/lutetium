use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::actor::{ActorContext, FromContext};
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::{PersistenceId, SequenceId, Version};
use crate::persistence::{Event, SelectionCriteria};
use crate::persistence::context::PersistContext;
use crate::system::ExtensionMissingError;

/// Trait that summarizes the database process for the [`PersistenceActor`](crate::persistence::actor::PersistenceActor) to store Events.
/// 
/// The data is **sequential** and assumes the use of IDs, such as `MySQL` using `AUTO_INCREMENT` for example.
/// However, I don't expect it to be registered as a Primary Key. 
/// It is only advisable to make adjustments against distribution, such as preparing a secondary index.
/// 
/// See [`SequenceId`](SequenceId) for ID specifications.
#[async_trait::async_trait]
pub trait JournalProvider: 'static + Sync + Send {
    async fn insert(&self, id: &PersistenceId, version: &Version, seq: &SequenceId, msg: JournalPayload) -> Result<(), PersistError>;
    async fn select_one(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<JournalPayload>, PersistError>;
    async fn select_many(&self, id: &PersistenceId, version: &Version, criteria: SelectionCriteria) -> Result<Option<BTreeSet<JournalPayload>>, PersistError>;
}

pub struct JournalProtocol(Arc<dyn JournalProvider>);

impl JournalProtocol {
    pub fn new(provider: impl JournalProvider) -> JournalProtocol {
        Self(Arc::new(provider))
    }
    
    pub async fn write_to_latest<E: Event>(&self, id: &PersistenceId, version: &Version, seq: SequenceId, event: &E) -> Result<(), PersistError> {
        self.0.insert(id, version, &seq, JournalPayload {
            seq, 
            key: E::REGISTRY_KEY, 
            bytes: event.as_bytes()? 
        }).await
    }
    
    pub async fn read(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<JournalPayload>, PersistError> {
        self.0.select_one(id, version, seq).await
    }
    
    pub async fn read_to(&self, id: &PersistenceId, version: &Version, criteria: SelectionCriteria) -> Result<Option<BTreeSet<JournalPayload>>, PersistError> {
        self.0.select_many(id, version, criteria).await
    }
    
    pub async fn read_to_latest(&self, id: &PersistenceId, version: &Version) -> Result<Option<BTreeSet<JournalPayload>>, PersistError> {
        self.0.select_many(id, version, SelectionCriteria::latest()).await
    }
}

impl Clone for JournalProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JournalPayload {
    pub seq: SequenceId,
    pub key: &'static str,
    pub bytes: Vec<u8>
}

impl JournalPayload {
    pub fn key(&self) -> &'static str {
        self.key
    }
}

impl PartialOrd<Self> for JournalPayload {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for JournalPayload {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seq.cmp(&other.seq)
    }
}


#[async_trait::async_trait]
impl FromContext<PersistContext> for JournalProtocol {
    type Rejection = ExtensionMissingError;
    async fn from_context(ctx: &mut PersistContext) -> Result<Self, Self::Rejection> {
        ctx.system()
            .extension()
            .get::<JournalProtocol>()
            .ok_or(ExtensionMissingError {
                module: "JournalProtocol"
            })
            .cloned()
    }
}
