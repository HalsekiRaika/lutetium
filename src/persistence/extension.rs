use std::sync::Arc;
use crate::actor::{Context, FromContext};
use crate::persistence::errors::PersistError;
use crate::persistence::identifier::PersistenceId;

#[async_trait::async_trait]
pub trait PersistenceProvider: 'static + Sync + Send {
    async fn create(&self, bin: Vec<u8>) -> Result<(), PersistError>;
    async fn select(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError>;
    async fn delete(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError>;
}

pub struct PersistenceModule(Arc<dyn PersistenceProvider>);

impl PersistenceModule {
    pub fn new(provider: impl PersistenceProvider) -> PersistenceModule {
        Self(Arc::new(provider))
    }
}

impl Clone for PersistenceModule {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl FromContext for PersistenceModule {
    type Rejection = PersistError;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        ctx.system()
            .ext
            .get::<PersistenceModule>()
            .ok_or_else(|| PersistError::Missing)
            .cloned()
    }
}