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

impl Context {
    pub fn track_with_system(system: ActorSystem) -> Self {
        Self { system, state: RunningState::default() }
    }
}

impl Context {
    pub fn shutdown(&mut self) {
        self.state.switch(|prev| { *prev = State::Shutdown });
    }

    pub fn state(&self) -> &RunningState {
        &self.state
    }
    
    pub fn system(&self) -> &ActorSystem {
        &self.system
    }
}

#[async_trait::async_trait]
pub trait FromContext: 'static + Sync + Send + Sized {
    type Rejection;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection>;
}