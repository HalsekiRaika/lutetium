use std::sync::Arc;

use crate::actor::{Context, FromContext};
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::PersistenceId;
use crate::persistence::SnapShot;
use crate::system::ExtensionMissingError;

#[async_trait::async_trait]
pub trait SnapShotProvider: 'static + Sync + Send {
    async fn insert(&self, id: &PersistenceId, payload: SnapShotPayload) -> Result<(), PersistError>;
    async fn select(&self, id: &PersistenceId) -> Result<Option<SnapShotPayload>, PersistError>;
    async fn delete(&self, id: &PersistenceId) -> Result<SnapShotPayload, PersistError>;
}

pub struct SnapShotProtocol(Arc<dyn SnapShotProvider>);

impl SnapShotProtocol {
    pub fn new(provider: impl SnapShotProvider) -> SnapShotProtocol {
        Self(Arc::new(provider))
    }


    pub async fn insert<S: SnapShot>(&self, id: &PersistenceId, snapshot: &S) -> Result<(), PersistError> {
        let payload = SnapShotPayload {
            id: id.clone(),
            key: S::REGISTRY_KEY,
            bytes: snapshot.as_bytes()?,
        };
        self.0.insert(id, payload).await
    }

    pub async fn select(&self, id: &PersistenceId) -> Result<Option<SnapShotPayload>, PersistError> {
        self.0.select(id).await
    }

    pub async fn delete(&self, id: &PersistenceId) -> Result<SnapShotPayload, PersistError> {
        self.0.delete(id).await
    }
}

impl Clone for SnapShotProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}


#[derive(Debug, Clone)]
pub struct SnapShotPayload {
    id: PersistenceId,
    key: &'static str,
    pub(crate) bytes: Vec<u8>
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
impl FromContext for SnapShotProtocol {
    type Rejection = ExtensionMissingError;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        ctx.system()
            .ext
            .get::<SnapShotProtocol>()
            .ok_or(ExtensionMissingError {
                module: "SnapShotProtocol"
            })
            .cloned()
    }
}
