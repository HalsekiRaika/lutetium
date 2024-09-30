use crate::persistence::context::PersistContext;
use crate::persistence::errors::{DeserializeError, SerializeError};

pub trait SnapShot: 'static + Sync + Send + Sized {
    const REGISTRY_KEY: &'static str;
    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError>;
    fn from_bytes(bin: &[u8]) -> Result<Self, DeserializeError>;
}

#[async_trait::async_trait]
pub trait RecoverSnapShot<S: SnapShot = Self>: 'static + Sync + Send + Sized {
    async fn recover_snapshot(this: &mut Option<Self>, snapshot: S, ctx: &mut PersistContext);
}
