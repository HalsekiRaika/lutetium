use tracing::Instrument;
use crate::actor::{Actor, Context, IntoActor};
use crate::actor::refs::{ActorRef, Applier};
use crate::errors::ActorError;
use crate::identifier::{ActorId, IntoActorId};

pub struct RunningActor<A: Actor> {
    pub id: ActorId,
    pub refs: ActorRef<A>
}

#[async_trait::async_trait]
pub trait Runnable
    where Self: IntoActor
{
    async fn run(self, id: impl IntoActorId, ctx: &mut Context) -> Result<RunningActor<Self::Actor>, ActorError> {
        let id = id.into_actor_id();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<Self::Actor>>>();
        let mut actor = self.into_actor();

        let refs = ActorRef::new(tx);

        let ctx = ctx.inherit();

        tokio::spawn(async move {
            let mut ctx = ctx; // moved
            let mut actor = actor;

            if let Err(e) = actor.activate(&mut ctx).await {
                tracing::error!(name: "on_activate", "An error occurred: {}", e);
                return;
            }

            while let Some(msg) = rx.recv().await {
                if let Err(e) = msg.apply(&mut actor, &mut ctx).await {
                    tracing::error!(name: "on_received", "An error occurred: {}", e);
                }

                if ctx.running_state().available_shutdown() {
                    break;
                }
            }
        }.instrument(tracing::info_span!("actor", id = %id)));


        Ok(RunningActor { id, refs })
    }
}
