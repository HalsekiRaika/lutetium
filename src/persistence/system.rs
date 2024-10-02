use std::future::Future;

use crate::actor::refs::ActorRef;
use crate::actor::{Actor, ActorContext};
use crate::errors::ActorError;
use crate::identifier::{IntoActorId, ToActorId};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::fixture::Fixable;
use crate::persistence::identifier::ToPersistenceId;
use crate::persistence::mapping::RecoveryMapping;
use crate::system::{ActorSystem, Behavior, LutetiumActorSystem};

#[async_trait::async_trait(?Send)]
pub trait PersistSystemExt: LutetiumActorSystem {
    async fn spawn_with_recovery<A>(&self, id: &impl ToPersistenceId, recoverable: Option<A>) -> Result<ActorRef<A>, ActorError>
        where A: PersistenceActor + RecoveryMapping;
    async fn find_or_spawn_with_recovery<A, I: ToActorId, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError>
        where A: PersistenceActor + RecoveryMapping,
              Fut: Future<Output = Option<A>> + Sync + Send;
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

    async fn find_or_spawn_with_recovery<A, I, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError>
    where
        I: ToActorId,
        A: PersistenceActor + RecoveryMapping,
        Fut: Future<Output = Option<A>> + Sync + Send
    {
        let actor_id = id.to_actor_id();
        let refs = match self.registry.find(&actor_id).await {
            Some((_, refs)) => refs.downcast::<A>()?,
            None => {
                let persistence_id = id.to_actor_id().to_persistence_id();
                let actor = or_nothing(id).await;
                self.spawn_with_recovery(&persistence_id, actor).await?
            }
        };
        
        Ok(refs)
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