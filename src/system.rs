mod extension;
mod lifecycle;
mod registry;

pub use self::{
    extension::*,
    lifecycle::*,
    registry::*,
};

use std::future::Future;
use std::sync::Arc;

use crate::actor::refs::ActorRef;
use crate::actor::{Actor, ActorContext, TryIntoActor};
use crate::errors::ActorError;
use crate::identifier::{IntoActorId, ToActorId};

pub struct ActorSystem {
    ext: Arc<Extensions>,
    registry: Registry
}

#[async_trait::async_trait(?Send)]
pub trait LutetiumActorSystem: 'static + Sync + Send {
    async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError>;
    async fn try_spawn<T: TryIntoActor>(&self, id: T::Identifier, into: T) -> Result<Result<ActorRef<T::Actor>, ActorError>, T::Rejection>;
    async fn shutdown(&self, id: impl ToActorId) -> Result<(), ActorError>;
    async fn shutdown_all(&self) -> Result<(), ActorError>;
    async fn find<A: Actor>(&self, id: impl ToActorId) -> Result<ActorRef<A>, ActorError>;
    async fn find_or<A: Actor, I: ToActorId, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError> where Fut: Future<Output = A> + 'static + Sync + Send;
}

#[async_trait::async_trait(?Send)]
impl LutetiumActorSystem for ActorSystem {
    async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError> {
        let behavior = Factory::create(actor, self.clone());
        let registered = self.registry
            .register(id.into_actor_id(), behavior)
            .await?;
        Ok(registered)
    }
    
    async fn try_spawn<T: TryIntoActor>(&self, id: T::Identifier, into: T) -> Result<Result<ActorRef<T::Actor>, ActorError>, T::Rejection> {
        let (id, actor) = into.try_into_actor(id)?;
        Ok(self.spawn(id, actor).await)
    }
    
    async fn shutdown(&self, id: impl ToActorId) -> Result<(), ActorError> {
        self.registry
            .deregister(&id.to_actor_id())
            .await
    }
    
    async fn shutdown_all(&self) -> Result<(), ActorError> {
        self.registry
            .shutdown_all()
            .await
    }
    
    async fn find<A: Actor>(&self, id: impl ToActorId) -> Result<ActorRef<A>, ActorError> {
        let id = id.to_actor_id();
        let Some((_, actor)) = self.registry.find(&id).await else {
            return Err(ActorError::NotFoundActor { id })
        };
        let refs = actor.downcast::<A>()?;
        Ok(refs)
    }
    
    async fn find_or<A: Actor, I: ToActorId, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError> 
        where Fut: Future<Output = A> + 'static + Sync + Send
    {
        let i = id.to_actor_id();
        match self.registry.find(&i).await {
            Some((_, actor)) => {
                actor.downcast::<A>()
            },
            None => {
                let actor = or_nothing(id).await;
                self.spawn(i, actor).await
            }
        }
    }
}

impl ActorSystem {
    pub fn builder() -> SystemBuilder {
        SystemBuilder {
            ext: Default::default(),
        }
    }
}

impl ActorSystem {
    pub fn extension(&self) -> &Arc<Extensions> {
        &self.ext
    }
}

impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self { 
            ext: Arc::clone(&self.ext),
            registry: self.registry.clone(),
        }
    }
}

pub(crate) struct Factory;

impl Factory {
    pub fn create<A: Actor>(actor: A, system: ActorSystem) -> Behavior<A> {
        Behavior { actor, ctx: A::Context::track_with_system(system) }
    }
}

pub struct Behavior<A: Actor> {
    actor: A,
    ctx: A::Context
}

pub struct SystemBuilder {
    ext: Extensions
}

impl SystemBuilder {
    pub fn extension(&mut self, procedure: impl FnOnce(&mut Extensions)) -> &mut Self {
        procedure(&mut self.ext);
        self
    }
    
    pub fn build(self) -> ActorSystem {
        ActorSystem {
            ext: Arc::new(self.ext),
            registry: Registry::default(),
        }
    }
}