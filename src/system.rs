mod extension;
mod lifecycle;
mod registry;

pub use self::{
    extension::*,
    lifecycle::*,
    registry::*,
};

use std::sync::Arc;
use std::future::Future;

use crate::actor::refs::ActorRef;
use crate::actor::{Actor, Context, TryIntoActor};
use crate::errors::ActorError;
use crate::identifier::{IntoActorId, ToActorId};

pub struct ActorSystem {
    ext: Arc<Extensions>,
    registry: Registry
}

impl ActorSystem {
    pub async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError> {
        let behavior = Factory::create(actor, self.clone());
        let registered = self.registry
            .register(id.into_actor_id(), behavior)
            .await?;
        Ok(registered)
    }
    
    pub async fn shutdown(&self, id: impl ToActorId) -> Result<(), ActorError> {
        self.registry
            .deregister(&id.to_actor_id())
            .await
    }
    
    pub async fn shutdown_all(&self) -> Result<(), ActorError> {
        self.registry
            .shutdown_all()
            .await
    }
    
    pub async fn find<A: Actor>(&self, id: impl ToActorId) -> Result<ActorRef<A>, ActorError> {
        let id = id.to_actor_id();
        let Some((_, actor)) = self.registry.find(&id).await else {
            return Err(ActorError::NotFoundActor { id })
        };
        let refs = actor.downcast::<A>()?;
        Ok(refs)
    }
    
    pub async fn find_or<A: Actor, I: ToActorId, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError> 
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
        Behavior { actor, ctx: Context::track_with_system(system) }
    }
}

pub struct Behavior<A: Actor> {
    actor: A,
    ctx: Context
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