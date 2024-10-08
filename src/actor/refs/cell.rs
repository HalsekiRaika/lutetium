use std::sync::Arc;
use crate::actor::RunningState;

pub struct ActorCell(pub(crate) Arc<InnerCell>);

pub(crate) struct InnerCell {
    pub(crate) running_state: RunningState
}

impl Clone for ActorCell {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}