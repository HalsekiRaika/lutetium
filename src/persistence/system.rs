use crate::actor::refs::ActorRef;
use crate::actor::{Actor, ActorContext};
use crate::errors::ActorError;
use crate::identifier::IntoActorId;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::fixture::Fixable;
use crate::persistence::identifier::ToPersistenceId;
use crate::persistence::mapping::RecoveryMapping;
use crate::system::{ActorSystem, Behavior, LutetiumActorSystem};

#[async_trait::async_trait(?Send)]
pub trait PersistSystemExt: LutetiumActorSystem {
    async fn spawn_with_recovery<A>(&self, id: &impl ToPersistenceId, recoverable: Option<A>) -> Result<ActorRef<A>, ActorError>
        where A: PersistenceActor + RecoveryMapping;
}

#[async_trait::async_trait(?Send)]
impl PersistSystemExt for ActorSystem {
    async fn spawn_with_recovery<A>(&self, id: &impl ToPersistenceId, recoverable: Option<A>) -> Result<ActorRef<A>, ActorError>
    where
        A: PersistenceActor + RecoveryMapping,
    {
        let id = id.to_persistence_id();
        let RecoverableBehavior { mut recoverable, mut context } = Factory::create(recoverable, self.clone());
        let fixture = A::recover(&id, &mut context).await
            .map_err(|e| ActorError::External(Box::new(e)))?;
        
        fixture.apply(&mut recoverable, &mut context).await
            .map_err(|e| ActorError::External(Box::new(e)))?;

        let recovered = recoverable.ok_or(ActorError::FailedActivation {
            reason: "recover",
            id: id.to_string(),
        })?;
        
        let behavior = Behavior::new(recovered, context);
        
        let registered = self.registry
            .register(id.into_actor_id(), behavior)
            .await?;
        
        Ok(registered)
    }
}

struct RecoverableBehavior<A: Actor> {
    recoverable: Option<A>,
    context: A::Context
}

struct Factory;

impl Factory {
    pub fn create<A: Actor>(recoverable: Option<A>, system: ActorSystem) -> RecoverableBehavior<A> {
        RecoverableBehavior { recoverable, context: A::Context::track_with_system(system) }
    }
}