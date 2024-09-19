use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::actor::{Context, FromContext};
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::{PersistenceId, SequenceId};
use crate::persistence::{Event, SelectionCriteria};
use crate::system::ExtensionMissingError;

/// Trait that summarizes the database process for the [`PersistenceActor`](crate::persistence::actor::PersistenceActor) to store Events.
/// 
/// The data is **sequential** and assumes the use of IDs, such as `MySQL` using `AUTO_INCREMENT` for example.
/// See [`SequenceId`](crate::persistence::identifier::SequenceId) for ID specifications.
#[async_trait::async_trait]
pub trait JournalProvider: 'static + Sync + Send {
    async fn insert(&self, id: &PersistenceId, msg: JournalPayload) -> Result<(), PersistError>;
    async fn select_one(&self, id: &PersistenceId, seq: SequenceId) -> Result<JournalPayload, PersistError>;
    async fn select_many(&self, id: &PersistenceId, criteria: SelectionCriteria) -> Result<BTreeSet<JournalPayload>, PersistError>;
}

pub struct JournalProtocol(Arc<dyn JournalProvider>);

impl JournalProtocol {
    pub fn new(provider: impl JournalProvider) -> JournalProtocol {
        Self(Arc::new(provider))
    }
    
    pub async fn insert<E: Event>(&self, id: &PersistenceId, event: &E) -> Result<(), PersistError> {
        todo!()
    }
    
    pub async fn select_one(&self, id: &PersistenceId, seq: SequenceId) -> Result<JournalPayload, PersistError> {
        todo!()
    }
    
    pub async fn select_many(&self, id: &PersistenceId, criteria: SelectionCriteria) -> Result<JournalPayload, PersistError> {
        todo!()
    }
}

impl Clone for JournalProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JournalPayload {
    seq: SequenceId,
    key: &'static str,
    bytes: Vec<u8>
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
