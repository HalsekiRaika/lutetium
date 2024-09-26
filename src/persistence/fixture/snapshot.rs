use tokio::time::Instant;
use crate::actor::FromContext;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::context::PersistContext;
use crate::persistence::errors::RecoveryError;
use crate::persistence::extension::SnapShotProtocol;
use crate::persistence::fixture::{Fixable, FixtureParts};
use crate::persistence::identifier::PersistenceId;
use crate::persistence::mapping::{RecoverMapping, RecoveryMapping};

pub struct FixtureSnapShot<A: PersistenceActor>(Option<FixtureParts<A>>);

impl<A: RecoveryMapping> FixtureSnapShot<A> {
    pub async fn create(id: &PersistenceId, ctx: &mut PersistContext) -> Result<Self, RecoveryError> {
        let mapping = RecoverMapping::<A>::create();
        
        if mapping.is_snapshot_map_empty() {
            tracing::trace!("journal recovery disabled");
            return Ok(Self::disable())
        }
        
        let provider = SnapShotProtocol::from_context(ctx).await?;

        let Some(payload) = provider.read_latest(id).await? else {
            tracing::trace!("snapshot recovery emptiness");
            return Ok(Self(None))
        };
        
        let Some(handle) = mapping.snapshot().find(payload.key()) else { 
            return Err(RecoveryError::NotCompatible(payload.key()))
        };
        
        
        Ok(Self(Some(FixtureParts::new(payload.seq, payload.bytes, handle))))
    }
    
    pub fn disable() -> Self {
        Self(None)
    }
    
    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor> Fixable<A> for FixtureSnapShot<A> {
    async fn apply(self, actor: &mut A, ctx: &mut PersistContext) -> Result<(), RecoveryError> {
        let Some(fixture) = self.0 else {
            return Ok(())
        };
        
        let now = Instant::now();
        
        fixture.refs.apply(actor, fixture.bytes, ctx).await?;
        *ctx.mut_sequence() = fixture.seq;
        tracing::trace!("snapshot recovered! {}ms", now.elapsed().as_millis());
        
        Ok(())
    }
}
