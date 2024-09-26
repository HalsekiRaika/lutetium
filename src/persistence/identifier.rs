use std::fmt::{Display, Formatter};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::identifier::ActorId;

#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
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

impl<T: ToString + Sync + Send + 'static> IntoPersistenceId for T {
    fn into_persistence_id(self) -> PersistenceId {
        PersistenceId::new(self.to_string())
    }
}

impl<T: ToString + Sync + Send + 'static> ToPersistenceId for T {
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


#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct SequenceId(Arc<i64>);

impl SequenceId {
    pub fn new(seq: i64) -> SequenceId {
        Self(Arc::new(seq))
    }
    
    pub fn min() -> SequenceId {
        Self(Arc::new(i64::MIN))
    }
    
    pub fn max() -> SequenceId {
        Self(Arc::new(i64::MAX))
    }
    
    pub async fn next(&self) -> SequenceId {
        Self::new(*self.0 + 1)
    }
}

impl From<SequenceId> for i64 {
    fn from(value: SequenceId) -> Self {
        *value.0
    }
}

impl AsRef<i64> for SequenceId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}