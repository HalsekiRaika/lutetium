use std::sync::Arc;
use tracing::Instrument;
use crate::actor::{Actor, ActorContext};
use crate::actor::refs::{ActorCell, ActorRef, Applier, InnerCell};
use crate::errors::ActorError;
use crate::system::Behavior;
use crate::system::registry::Registry;

pub(crate) struct LifeCycle;

impl LifeCycle {
    pub async fn spawn<A: Actor>(registry: Registry, behavior: Behavior<A>) -> Result<ActorRef<A>, ActorError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();
        let Behavior { mut actor, mut ctx } = behavior;
        let cell = ActorCell(Arc::new(InnerCell {
            running_state: ctx.state().clone()
        }));
        
        let refs = ActorRef::new(cell, tx);

        actor.activate(&mut ctx).await?;

        let span = ctx.id().to_owned();
        
        tokio::spawn(async move {

            let mut ctx = ctx;
            let mut actor = actor;
            let registry = registry;

            tracing::trace!("resource moved to tokio thread lifecycle");
            
            while let Some(payload) = rx.recv().await {
                if let Err(e) = payload.apply(&mut actor, &mut ctx).await {
                    tracing::error!("{}", e)
                }

                if ctx.state().available_shutdown().await {
                    tracing::warn!("shutdown");
                    break;
                }
            }
            tracing::trace!("lifecycle ended.");
            
            if let Err(e) = registry.deregister(ctx.id()).await {
                tracing::error!("{}", e);
            }
        }.instrument(tracing::info_span!("{}", actor_id = %span)));

        Ok(refs)
    }
}
