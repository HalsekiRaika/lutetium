use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use async_trait::async_trait;

use tracing::Instrument;

use crate::actor::{Actor, Context, Handler, Message};
use crate::actor::refs::{ActorRef, AnyRef, Applier, DynRef};
use crate::actor::refs::RegularAction;
use crate::errors::ActorError;
use crate::identifier::{ActorId, IntoActorId, ToActorId};
use crate::system::System;

pub struct Supervisor {
    pub(crate) actors: HashMap<ActorId, AnyRef>
}

pub struct SupervisorRef(ActorRef<Supervisor>);

impl Supervisor {
    pub(crate) fn new() -> Supervisor {
        Self { actors: HashMap::new() }
    }
    
    pub fn activate(mut self, system: Arc<System>) -> SupervisorRef {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<Supervisor>>>();

        let refs = ActorRef::new(tx);

        let supervisor_ref = SupervisorRef(refs);

        let ctx = Context::new(system, supervisor_ref.clone());
        
        tokio::spawn(async move {
            let mut ctx = ctx;
            
            match Actor::activate(&mut self, &mut ctx).await {
                Ok(_) => {
                    while let Some(payload) = rx.recv().await {
                        if let Err(e) = payload.apply(&mut self, &mut ctx).await {
                            tracing::error!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        
        supervisor_ref
    }
}

impl SupervisorRef {
    pub async fn spawn<A: Actor>(&self, id: impl IntoActorId, actor: A) -> Result<ActorRef<A>, ActorError> {
        self.0.ask(RunnableActor { id: id.into_actor_id(), actor }).await?
    }
    
    pub async fn shutdown(&self, id: impl IntoActorId) -> Result<(), ActorError> {
        self.0.tell(ShutdownActor { id: id.into_actor_id() }).await?
    }
    
    pub async fn shutdown_all(&self) -> Result<(), ActorError> {
        self.0.tell(ShutdownAll).await?
    }

    pub async fn find<A: Actor>(&self, id: impl IntoActorId) -> Result<Option<ActorRef<A>>, ActorError> {
        self.0.ask(FindActor { id: id.into_actor_id(), _mark: PhantomData }).await?
    }

    pub async fn find_or<A: Actor, I: ToActorId, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError>
        where Fut: Future<Output=A> + 'static + Send,
    {
        let actor_id = id.to_actor_id();
        match self.0.ask(FindActor { id: actor_id.clone(), _mark: PhantomData }).await?? {
            Some(actor) => Ok(actor),
            None => {
                let data = or_nothing(id).await;
                
                self.0.ask(RunnableActor { id: actor_id, actor: data }).await?
            }
        }
    }
}

impl Clone for SupervisorRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl Actor for Supervisor {
    async fn activate(&mut self, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::info!("supervisor activate.");
        Ok(())
    }
}

#[async_trait]
impl<A: Actor> Handler<RunnableActor<A>> for Supervisor {
    type Accept = ActorRef<A>;
    type Rejection = ActorError;

    async fn call(&mut self, mut msg: RunnableActor<A>, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();

        let refs = ActorRef::new(tx);

        if self.actors.insert(msg.id.clone(), refs.clone().into()).is_some() {
            return Err(ActorError::AlreadySpawned { id: msg.id })
        }

        let ctx = ctx.inherit();

        tokio::spawn(async move {
            let mut ctx = ctx;
            
            match msg.actor.activate(&mut ctx).await {
                Ok(_) => {
                    tracing::info!("spawned.");
                    while let Some(payload) = rx.recv().await {
                        if let Err(e) = payload.apply(&mut msg.actor, &mut ctx).await {
                            tracing::error!("{}", e);
                        }

                        if ctx.running_state().available_shutdown() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(name: "activation", "{}", e);
                }
            }
            
            tracing::warn!("shutdown.");
        }.instrument(tracing::info_span!("actor", id = %msg.id)));

        Ok(refs)
    }
}

#[async_trait]
impl Handler<ShutdownActor> for Supervisor {
    type Accept = ();
    type Rejection = ActorError;

    async fn call(&mut self, msg: ShutdownActor, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        let Some((_, actor)) = self.actors.iter_mut().find(|(id, _)| msg.id.eq(id)) else {
            return Err(ActorError::NotFoundActor {
                id: msg.id,
            })
        };
        
        actor.shutdown().await?;
        
        Ok(())
    }
}

#[async_trait]
impl<A: Actor> Handler<FindActor<A>> for Supervisor {
    type Accept = Option<ActorRef<A>>;
    type Rejection = ActorError;

    async fn call(&mut self, msg: FindActor<A>, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        self.actors.iter()
            .find(|(id, _)| msg.id.eq(id))
            .map(|(_, refs)| refs.clone())
            .map(|refs| refs.downcast::<A>())
            .transpose()
    }
}

#[async_trait]
impl Handler<ShutdownAll> for Supervisor {
    type Accept = ();
    type Rejection = ActorError;
    
    async fn call(&mut self, _msg: ShutdownAll, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        tracing::warn!("Shutdown the Actor under the current Supervisor.");
        for actor in self.actors.values() {
            actor.shutdown().await?;
        }
        
        Ok(())
    }
}

pub struct RunnableActor<A: Actor> {
    id: ActorId,
    actor: A
}

impl<A: Actor> Message for RunnableActor<A> {}

pub struct FindActor<A: Actor> {
    id: ActorId,
    _mark: PhantomData<A>
}

impl<A: Actor> Message for FindActor<A> {}

pub struct ShutdownActor {
    id: ActorId
}

impl Message for ShutdownActor {}

pub struct ShutdownAll;

impl Message for ShutdownAll {}
