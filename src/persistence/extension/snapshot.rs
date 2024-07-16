use std::sync::Arc;

use crate::actor::{Context, FromContext};
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::PersistenceId;
use crate::persistence::SnapShot;

#[async_trait::async_trait]
pub trait PersistenceProvider: 'static + Sync + Send {
    async fn insert(&self, id: &PersistenceId, bin: Vec<u8>) -> Result<(), PersistError>;
    async fn select(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError>;
    async fn delete(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError>;
}

pub struct SnapShotProtocol(Arc<dyn PersistenceProvider>);

impl SnapShotProtocol {
    pub fn new(provider: impl PersistenceProvider) -> SnapShotProtocol {
        Self(Arc::new(provider))
    }


    pub async fn insert<S: SnapShot>(&self, id: &PersistenceId, snapshot: &S) -> Result<(), PersistError> {
        self.0.insert(id, snapshot.as_bytes()?).await
    }

    pub async fn select<S: SnapShot>(&self, id: &PersistenceId) -> Result<S, PersistError> {
        let bin = self.0.select(id).await?;
        S::from_bytes(&bin)
    }

    pub async fn delete<S: SnapShot>(&self, id: &PersistenceId) -> Result<S, PersistError> {
        let bin = self.0.delete(id).await?;
        S::from_bytes(&bin)
    }
}

impl Clone for SnapShotProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait::async_trait]
impl FromContext for SnapShotProtocol {
    type Rejection = PersistError;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        ctx.system()
            .ext
            .get::<SnapShotProtocol>()
            .ok_or_else(|| PersistError::Missing)
            .cloned()
    }
}
