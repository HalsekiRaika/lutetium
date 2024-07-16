use std::ops::Deref;
use std::sync::Arc;
use crate::actor::Context;

pub use self::extension::*;
pub use self::root::*;
pub use self::supervisor::*;

mod extension;
mod root;
mod supervisor;

pub struct ActorSystem {
    pub(crate) root: Root,
    pub(crate) system: Arc<System> 
}

impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            system: Arc::clone(&self.system)
        }
    }
}

impl ActorSystem {
    pub fn builder() -> System {
        System::default()
    }
    
    pub fn root_context(&self) -> Context {
        Context::new(self.system.clone(), self.root.clone().into())
    }
}

impl Deref for ActorSystem {
    type Target = SupervisorRef;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}


pub struct System {
    pub(crate) ext: Extensions
}

impl System {
    pub fn extension(&mut self, f: impl FnOnce(&mut Extensions)) {
        f(&mut self.ext)
    }
    
    pub fn build(self) -> ActorSystem {
        let system = Arc::new(self);
        let root = Supervisor::new();
        ActorSystem {
            root: Root::new(root, Arc::clone(&system)),
            system
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for System {
    fn default() -> Self {
        Self {
            ext: Extensions::default()
        }
    }
}
