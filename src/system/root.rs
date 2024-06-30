use std::ops::Deref;
use std::sync::Arc;

use crate::system::{Supervisor, SupervisorRef, System};

pub struct Root {
    refs: SupervisorRef
}

impl Root {
    pub fn new(supervisor: Supervisor, system: Arc<System>) -> Root {
        Self { refs: supervisor.activate(system) }
    }
    
    pub(crate) fn inherit(as_root: SupervisorRef) -> Root {
        Self { refs: as_root }
    }
}

impl Clone for Root {
    fn clone(&self) -> Self {
        Self { refs: self.refs.clone() }
    }
}

impl Deref for Root {
    type Target = SupervisorRef;
    fn deref(&self) -> &Self::Target {
        &self.refs
    }
}