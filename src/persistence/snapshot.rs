use crate::actor::Context;
use crate::persistence::errors::PersistError;

pub trait SnapShot: 'static + Sync + Send + Sized {
    fn as_bytes(&self) -> Result<Vec<u8>, PersistError>;
    fn from_bytes(bin: &[u8]) -> Result<Self, PersistError>;
}

#[async_trait::async_trait]
pub trait RecoverSnapShot<S: SnapShot = Self>: 'static + Sync + Send {
    async fn recover_snapshot(&mut self, snapshot: S, ctx: &mut Context);
}