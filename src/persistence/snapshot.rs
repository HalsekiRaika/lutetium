use crate::actor::{Context, FromContext};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::errors::{DeserializeError, RecoveryError, SerializeError};
use crate::persistence::identifier::PersistenceId;
use crate::persistence::recovery::FixtureParts;
use crate::persistence::SnapShotProtocol;

pub trait SnapShot: 'static + Sync + Send + Sized {
    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError>;
    fn from_bytes(bin: &[u8]) -> Result<Self, DeserializeError>;
}

#[async_trait::async_trait]
pub trait RecoverSnapShot<S: SnapShot = Self>: 'static + Sync + Send {
    async fn recover_snapshot(&mut self, snapshot: S, ctx: &mut Context);
}

pub struct FixtureSnapShot<A: PersistenceActor>(Option<FixtureParts<A>>);

impl<A: PersistenceActor> FixtureSnapShot<A> {
    pub async fn from_id_with_context(id: &PersistenceId, ctx: &mut Context) -> Result<Self, RecoveryError> {
        let provider = SnapShotProtocol::from_context(ctx).await?;
        
        let snapshot = provider.select_raw(&id).await;
        todo!()
    }
}

