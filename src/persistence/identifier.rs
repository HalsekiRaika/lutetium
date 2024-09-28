use crate::identifier::ActorId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

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


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct SequenceId(i64);

impl SequenceId {
    pub fn new(seq: i64) -> SequenceId {
        Self(seq)
    }
    
    pub fn min() -> SequenceId {
        Self(i64::MIN)
    }
    
    pub fn max() -> SequenceId {
        Self(i64::MAX)
    }
    
    pub fn incr(&mut self) {
        self.0 += 1
    }
    
    pub fn assign(&mut self, assign: SequenceId) {
        self.0 = assign.0
    }
}

impl From<SequenceId> for i64 {
    fn from(value: SequenceId) -> Self {
        value.0
    }
}

impl AsRef<i64> for SequenceId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Default for SequenceId {
    fn default() -> Self {
        Self::new(0)
    }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Version(&'static str);

impl Version {
    pub const fn new(version: &'static str) -> Version {
        Self(version)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        self.0
    }
}