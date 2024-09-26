use crate::actor::{RunningState, State};
use crate::system::ActorSystem;


/// A structure representing the current state of the managed Actor.
///
/// Since each Actor is unique, [`Clone`] is not implemented because it can be very confusing.
/// Instead, [Context::inherit] is defined to create a Context while inheriting a reference to ActorSystem and a reference to Supervisor.
pub struct Context {
    system: ActorSystem,
    state: RunningState
}

impl ActorContext for Context {
    fn track_with_system(system: ActorSystem) -> Self {
        Self { system, state: RunningState::default() }
    }

    fn shutdown(&mut self) {
        self.state.switch(|prev| { *prev = State::Shutdown });
    }
    
    fn state(&self) -> &RunningState {
        &self.state
    }

    fn system(&self) -> &ActorSystem {
        &self.system
    }
}

pub trait ActorContext: 'static + Sync + Send + Sized {
    fn track_with_system(system: ActorSystem) -> Self;
    fn shutdown(&mut self);
    fn state(&self) -> &RunningState;
    fn system(&self) -> &ActorSystem;
}

#[async_trait::async_trait]
pub trait FromContext<C: ActorContext>: 'static + Sync + Send + Sized {
    type Rejection;
    async fn from_context(ctx: &mut C) -> Result<Self, Self::Rejection>;
}