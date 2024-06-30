use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ActorId {
    id: Arc<str>,
}

impl ActorId {
    pub(crate) fn new(id: String) -> ActorId {
        Self { id: id.into() }
    }
}

impl Clone for ActorId {
    fn clone(&self) -> Self {
        Self { id: Arc::clone(&self.id) }
    }
}

impl Display for ActorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub trait IntoActorId {
    fn into_actor_id(self) -> ActorId;
}

pub trait ToActorId {
    fn to_actor_id(&self) -> ActorId;
}

impl<T: ToString + Sync + Send> IntoActorId for T {
    fn into_actor_id(self) -> ActorId {
        ActorId::new(self.to_string())
    }
}

impl<T: ToString + Sync + Send> ToActorId for T {
    fn to_actor_id(&self) -> ActorId {
        ActorId::new(self.to_string())
    }
}