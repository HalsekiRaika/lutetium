use crate::actor::{ActorContext, RunningState, State};
use crate::persistence::identifier::SequenceId;
use crate::system::ActorSystem;

pub struct PersistContext {
    system: ActorSystem,
    state: RunningState,
    sequence: SequenceId
}

impl PersistContext {
    pub fn sequence(&self) -> &SequenceId {
        &self.sequence
    }
    
    pub fn mut_sequence(&mut self) -> &mut SequenceId {
        &mut self.sequence
    }
}

impl ActorContext for PersistContext {
    fn track_with_system(system: ActorSystem) -> Self {
        Self { system, state: RunningState::default(), sequence: SequenceId::new(0) }
    }

    fn shutdown(&mut self) {
        self.state.switch(|prev| *prev = State::Shutdown)
    }

    fn state(&self) -> &RunningState {
        &self.state
    }

    fn system(&self) -> &ActorSystem {
        &self.system
    }
}