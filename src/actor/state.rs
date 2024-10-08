use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RunningState(Arc<RwLock<State>>);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum State {
    Active,
    Shutdown
}

impl RunningState {
    pub(crate) async fn switch<Fut>(&self, f: impl Fn(Arc<RwLock<State>>) -> Fut) 
        where Fut: Future<Output=()> + 'static + Sync + Send
    {
        f(Arc::clone(&self.0)).await;
    }
    
    pub async fn is_active(&self) -> bool {
        let read = self.0.read().await;
        read.eq(&State::Active) 
    }
    
    pub async fn available_shutdown(&self) -> bool {
        !self.is_active().await
    }
}

impl Clone for RunningState {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Default for RunningState {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(State::Active)))
    }
}