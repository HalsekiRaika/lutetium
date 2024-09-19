use crate::actor::{Context, FromContext};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::errors::RecoveryError;
use crate::persistence::fixture::{Fixable, FixtureParts};
use crate::persistence::identifier::PersistenceId;
use crate::persistence::mapping::{RecoverMapping, RecoveryMapping};
use crate::persistence::SnapShotProtocol;

pub struct FixtureSnapShot<A: PersistenceActor>(Option<FixtureParts<A>>);

impl<A: RecoveryMapping> FixtureSnapShot<A> {
    pub async fn create(id: &PersistenceId, ctx: &mut Context) -> Result<Self, RecoveryError> {
        let provider = SnapShotProtocol::from_context(ctx).await?;

        let Some(payload) = provider.select(id).await? else {
            return Ok(Self(None))
        };
        
        let mapping = RecoverMapping::<A>::create();
        
        let Some(handle) = mapping.snapshot().find(payload.key()) else { 
            return Err(RecoveryError::NotCompatible(payload.key()))
        };
        
        
        Ok(Self(Some(FixtureParts::new(payload.bytes, handle))))
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor> Fixable<A> for FixtureSnapShot<A> {
    async fn apply(self, actor: &mut A, ctx: &mut Context) -> Result<(), RecoveryError> {
        let Some(fixture) = self.0 else {
            return Ok(())
        };
        
        fixture.refs.apply(actor, fixture.bytes, ctx).await?;
        
        Ok(())
    }
}
