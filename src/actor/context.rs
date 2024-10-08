use crate::actor::{RunningState, State};
use crate::identifier::{ActorId, IntoActorId};
use crate::system::ActorSystem;


/// A structure representing the current state of the managed Actor.
///
/// Since each Actor is unique, [`Clone`] is not implemented because it can be very confusing.
/// Instead, [Context::inherit] is defined to create a Context while inheriting a reference to ActorSystem and a reference to Supervisor.
pub struct Context {
    id: ActorId,
    system: ActorSystem,
    state: RunningState
}

#[async_trait::async_trait]
impl ActorContext for Context {
    fn track_with_system(id: impl IntoActorId, system: ActorSystem) -> Self {
        Self { id: id.into_actor_id(), system, state: RunningState::default() }
    }
    
    fn id(&self) -> &ActorId {
        &self.id
    }

    //noinspection DuplicatedCode
    async fn shutdown(&self) {
        self.state.switch(|prev| async move { 
            let mut write = prev.write().await;
            *write = State::Shutdown;
        }).await;
    }
    
    fn state(&self) -> &RunningState {
        &self.state
    }

    fn system(&self) -> &ActorSystem {
        &self.system
    }
}

#[async_trait::async_trait]
pub trait ActorContext: 'static + Sync + Send + Sized {
    fn track_with_system(id: impl IntoActorId, system: ActorSystem) -> Self;

    fn id(&self) -> &ActorId;
    async fn shutdown(&self);
    fn state(&self) -> &RunningState;
    fn system(&self) -> &ActorSystem;
}

#[async_trait::async_trait]
pub trait FromContext<C: ActorContext>: 'static + Sync + Send + Sized {
    type Rejection;
    async fn from_context(ctx: &mut C) -> Result<Self, Self::Rejection>;
}