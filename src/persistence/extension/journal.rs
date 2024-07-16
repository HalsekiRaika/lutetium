use std::sync::Arc;
use crate::actor::{Context, FromContext};
use crate::persistence::errors::PersistError;

#[async_trait::async_trait]
pub trait JournalProvider: 'static + Sync + Send {
    async fn insert(&self, msg: Vec<u8>) -> Result<(), PersistError>;
    async fn select(&self, id: i32) -> Result<(), PersistError>;
    async fn delete(&self, id: i32) -> Result<(), PersistError>;
}

pub struct JournalProtocol(Arc<dyn JournalProvider>);

impl JournalProtocol {
    pub fn new(provider: impl JournalProvider) -> JournalProtocol {
        Self(Arc::new(provider))
    }
}

impl Clone for JournalProtocol {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait::async_trait]
impl FromContext for JournalProtocol {
    type Rejection = PersistError;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        ctx.system()
            .ext
            .get::<JournalProtocol>()
            .ok_or(PersistError::Missing)
            .cloned()
    }
}
