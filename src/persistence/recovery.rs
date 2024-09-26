use std::marker::PhantomData;

use crate::persistence::errors::RecoveryError;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::{Event, RecoverJournal, RecoverSnapShot, SnapShot};
use crate::persistence::context::PersistContext;

#[async_trait::async_trait]
pub(crate) trait Handler<A: PersistenceActor>: 'static + Sync + Send {
    async fn apply(&self, actor: &mut A, payload: Vec<u8>, ctx: &mut PersistContext) -> Result<(), RecoveryError>;
}


pub struct SnapShotResolver<A: PersistenceActor, S: SnapShot> {
    _actor: PhantomData<A>,
    _snapshot: PhantomData<S>
}

pub struct EventResolver<A: PersistenceActor, E: Event> {
    _actor: PhantomData<A>,
    _event: PhantomData<E>
}

impl<A: PersistenceActor, S: SnapShot> Default for SnapShotResolver<A, S> {
    fn default() -> Self {
        Self { _actor: PhantomData, _snapshot: PhantomData }
    }
}

impl<A: PersistenceActor, E: Event> Default for EventResolver<A, E> {
    fn default() -> Self {
        Self { _actor: PhantomData, _event: PhantomData }
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor, S: SnapShot> Handler<A> for SnapShotResolver<A, S> 
    where A: RecoverSnapShot<S>
{
    async fn apply(&self, actor: &mut A, payload: Vec<u8>, ctx: &mut PersistContext) -> Result<(), RecoveryError> {
        let decode = S::from_bytes(&payload)?;
        actor.recover_snapshot(decode, ctx).await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor, E: Event> Handler<A> for EventResolver<A, E> 
    where A: RecoverJournal<E>
{
    async fn apply(&self, actor: &mut A, payload: Vec<u8>, ctx: &mut PersistContext) -> Result<(), RecoveryError> {
        let decode = E::from_bytes(&payload)?;
        actor.recover_journal(decode, ctx).await;
        Ok(())
    }
}