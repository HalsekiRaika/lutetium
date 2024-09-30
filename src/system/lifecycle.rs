use crate::actor::{Actor, ActorContext};
use crate::actor::refs::{ActorRef, Applier};
use crate::errors::ActorError;
use crate::system::Behavior;

pub(crate) struct LifeCycle;

impl LifeCycle {
    pub async fn spawn<A: Actor>(behavior: Behavior<A>) -> Result<ActorRef<A>, ActorError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();
        let refs = ActorRef::new(tx);
        let Behavior { mut actor, mut ctx } = behavior;

        actor.activate(&mut ctx).await?;

        tokio::spawn(async move {

            let mut ctx = ctx;
            let mut actor = actor;

            tracing::trace!("resource moved to tokio thread lifecycle");
            
            while let Some(payload) = rx.recv().await {
                if let Err(e) = payload.apply(&mut actor, &mut ctx).await {
                    tracing::error!("{}", e)
                }

                if ctx.state().available_shutdown() {
                    tracing::warn!("shutdown");
                    break;
                }
            }
            tracing::trace!("lifecycle ended.");
        });

        Ok(refs)
    }
}
