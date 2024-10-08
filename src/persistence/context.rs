use crate::actor::{ActorContext, RunningState, State};
use crate::identifier::{ActorId, IntoActorId};
use crate::persistence::identifier::SequenceId;
use crate::system::ActorSystem;

pub struct PersistContext {
    id: ActorId,
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

#[async_trait::async_trait]
impl ActorContext for PersistContext {
    fn track_with_system(id: impl IntoActorId, system: ActorSystem) -> Self {
        Self { id: id.into_actor_id(), system, state: RunningState::default(), sequence: SequenceId::new(0) }
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