use std::sync::Arc;

use crate::actor::{RunningState, State};
use crate::system::{Root, SupervisorRef, System};

pub struct Context {
    root: Root,
    system: Arc<System>,
    running: RunningState,
}

impl Context {
    pub(crate) fn new(system: Arc<System>, root: SupervisorRef) -> Context {
        Self {
            root: Root::inherit(root),
            system,
            running: RunningState::default(),
        }
    }
    
    pub(crate) fn inherit(&self) -> Context {
        Self {
            root: self.root.clone(),
            system: self.system.clone(),
            running: RunningState::default()
        }
    }
}

impl Context {
    pub fn shutdown(&mut self) {
        self.running.switch(|prev| { *prev = State::Shutdown });
    }
}

impl Context {
    pub fn system(&self) -> Arc<System> {
        Arc::clone(&self.system)
    }
    
    pub fn root(&self) -> Root {
        self.root.clone()
    }
    
    pub(crate) fn running_state(&self) -> &RunningState {
        &self.running
    }
}

#[async_trait::async_trait]
pub trait FromContext: 'static + Sync + Send + Sized {
    type Rejection;
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection>;
}