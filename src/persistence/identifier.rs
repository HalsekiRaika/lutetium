use std::fmt::{Display, Formatter};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::identifier::ActorId;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct PersistenceId(Arc<str>);

impl PersistenceId {
    pub fn new(id: String) -> PersistenceId {
        Self(id.into())
    }
}

impl Clone for PersistenceId {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Display for PersistenceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait IntoPersistenceId: 'static + Sync + Send {
    fn into_persistence_id(self) -> PersistenceId;
}

pub trait ToPersistenceId: 'static + Sync + Send {
    fn to_persistence_id(&self) -> PersistenceId;
}

impl<T: ToString + Sync + Send> IntoPersistenceId for T {
    fn into_persistence_id(self) -> PersistenceId {
        PersistenceId::new(self.to_string())
    }
}

impl<T: ToString + Sync + Send> ToPersistenceId for T {
    fn to_persistence_id(&self) -> PersistenceId {
        PersistenceId::new(self.to_string())
    }
}

impl From<ActorId> for PersistenceId {
    fn from(value: ActorId) -> Self {
        Self(value.id)
    }
}

impl From<PersistenceId> for ActorId {
    fn from(value: PersistenceId) -> Self {
        Self { id: value.0 }
    }
}