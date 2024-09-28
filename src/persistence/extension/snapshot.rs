use std::sync::Arc;

use crate::actor::{ActorContext, FromContext};
use crate::persistence::context::PersistContext;
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::{PersistenceId, SequenceId, Version};
use crate::persistence::SnapShot;
use crate::system::ExtensionMissingError;

#[async_trait::async_trait]
pub trait SnapShotProvider: 'static + Sync + Send {
    async fn insert(&self, id: &PersistenceId, version: &Version, seq: &SequenceId, payload: SnapShotPayload) -> Result<(), PersistError>;
    async fn select(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<SnapShotPayload>, PersistError>;
}

pub struct SnapShotProtocol(Arc<dyn SnapShotProvider>);

impl SnapShotProtocol {
    pub fn new(provider: impl SnapShotProvider) -> SnapShotProtocol {
        Self(Arc::new(provider))
    }


    pub async fn write<S: SnapShot>(&self, id: &PersistenceId, version: &Version, seq: SequenceId, snapshot: &S) -> Result<(), PersistError> {
        let payload = SnapShotPayload {
            id: id.clone(),
            key: S::REGISTRY_KEY,
            seq,
            bytes: snapshot.as_bytes()?,
        };
        self.0.insert(id, version, &seq, payload).await
    }

    pub async fn read(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<SnapShotPayload>, PersistError> {
        self.0.select(id, version, seq).await
    }

    pub async fn read_latest(&self, id: &PersistenceId, version: &Version) -> Result<Option<SnapShotPayload>, PersistError> {
        self.0.select(id, version, &SequenceId::max()).await
    }
}

impl Clone for SnapShotProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


#[derive(Debug, Clone)]
pub struct SnapShotPayload {
    pub id: PersistenceId,
    pub key: &'static str,
    pub seq: SequenceId,
    pub bytes: Vec<u8>
}

impl SnapShotPayload {
    pub fn id(&self) -> &PersistenceId {
        &self.id
    }
    
    pub fn key(&self) -> &'static str {
        self.key
    }
    
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[async_trait::async_trait]
impl FromContext<PersistContext> for SnapShotProtocol {
    type Rejection = ExtensionMissingError;
    async fn from_context(ctx: &mut PersistContext) -> Result<Self, Self::Rejection> {
        ctx.system()
            .extension()
            .get::<SnapShotProtocol>()
            .ok_or(ExtensionMissingError {
                module: "SnapShotProtocol"
            })
            .cloned()
    }
}
