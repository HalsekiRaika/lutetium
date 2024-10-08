mod extension;
mod lifecycle;
mod registry;

pub use self::extension::*;

use std::future::Future;
use std::sync::Arc;

use crate::actor::refs::ActorRef;
use crate::actor::{Actor, ActorContext, FromMessage, Message, TryIntoActor};
use crate::errors::ActorError;
use crate::identifier::{IntoActorId, ToActorId};
use crate::system::registry::Registry;

pub struct ActorSystem {
    pub(crate) ext: Arc<Extensions>,
    pub(crate) registry: Registry
}

#[async_trait::async_trait]
pub trait LutetiumActorSystem: 'static + Sync + Send {
    async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError>;
    async fn spawn_from<A: Actor, M: Message>(&self, from: M) -> Result<Result<ActorRef<A>, ActorError>, A::Rejection>
        where A: FromMessage<M>;
    async fn try_spawn<A: Actor, T: TryIntoActor<A>>(&self, id: T::Identifier, into: T) -> Result<Result<ActorRef<A>, ActorError>, T::Rejection>;
    async fn shutdown(&self, id: &impl ToActorId) -> Result<(), ActorError>;
    async fn shutdown_all(&self) -> Result<(), ActorError>;
    async fn find<A: Actor>(&self, id: impl ToActorId) -> Result<ActorRef<A>, ActorError>;
    async fn find_or<A: Actor, I: ToActorId, Fn, Fut>(&self, id: I, or_nothing: Fn) -> Result<ActorRef<A>, ActorError> 
        where
            Fn: FnOnce(I) -> Fut + 'static + Sync + Send,
            Fut: Future<Output = A> + 'static + Sync + Send;
}

#[async_trait::async_trait]
impl LutetiumActorSystem for ActorSystem {
    async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError> {
        let id = id.into_actor_id();
        let behavior = Factory::create(actor, id.clone(), self.clone());
        let registered = self.registry
            .register(id, behavior)
            .await?;
        Ok(registered)
    }


    async fn spawn_from<A: Actor, M: Message>(&self, from: M) -> Result<Result<ActorRef<A>, ActorError>, A::Rejection>
        where A: FromMessage<M>
    {
        let mut ctx = A::Context::track_with_system("prepare", self.clone());
        let (id, actor) = A::once(from, &mut ctx).await?;
        let id = id.into_actor_id();
        let ctx = A::Context::track_with_system(id.clone(), self.clone());
        let behavior = Behavior::new(actor, ctx);
        let registered = self.registry
            .register(id, behavior)
            .await;
        Ok(registered)
    }
    
    async fn try_spawn<A: Actor,T: TryIntoActor<A>>(&self, id: T::Identifier, into: T) -> Result<Result<ActorRef<A>, ActorError>, T::Rejection> {
        let (id, actor) = into.try_into_actor(id)?;
        Ok(self.spawn(id, actor).await)
    }
    
    async fn shutdown(&self, id: &impl ToActorId) -> Result<(), ActorError> {
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
    
    async fn find_or<A: Actor, I: ToActorId, Fn, Fut>(&self, id: I, or_nothing: Fn) -> Result<ActorRef<A>, ActorError> 
        where 
            Fn: FnOnce(I) -> Fut + 'static + Sync + Send,
            Fut: Future<Output = A> + 'static + Sync + Send
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
    pub fn create<A: Actor>(actor: A, id: impl IntoActorId, system: ActorSystem) -> Behavior<A> {
        Behavior::new(actor, A::Context::track_with_system(id, system))
    }
}

pub(crate) struct Behavior<A: Actor> {
    actor: A,
    ctx: A::Context
}

impl<A: Actor> Behavior<A> {
    pub fn new(actor: A, ctx: A::Context) -> Self {
        Self { actor, ctx }
    }
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