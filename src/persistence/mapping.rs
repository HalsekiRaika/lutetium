use std::sync::Arc;
use std::any::TypeId;

use dashmap::DashMap;

use crate::persistence::actor::PersistenceActor;
use crate::persistence::recovery::{EventResolver, Handler, SnapShotResolver};
use crate::persistence::{Event, RecoverJournal, RecoverSnapShot, SnapShot};

pub trait RecoveryMapping: PersistenceActor {
    fn mapping(mapping: &mut RecoverMapping<Self>);
}

pub struct RecoverMapping<A: PersistenceActor> {
    snapshot: ResolveMapper<A>,
    event: ResolveMapper<A>,
}

pub struct ResolveMapper<A: PersistenceActor>(DashMap<RecoveryKey, Arc<dyn Handler<A>>>);

impl<A: PersistenceActor> Default for ResolveMapper<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: PersistenceActor> ResolveMapper<A> {
    pub(crate) fn find(&self, key: &str) -> Option<Arc<dyn Handler<A>>> {
        self.0.iter().find(|s| s.key().eq(key))
            .map(|r| Arc::clone(r.value()))
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct RecoveryKey(TypeId, &'static str);

impl RecoveryKey {
    pub fn new<T: 'static>(key: &'static str) -> RecoveryKey {
        Self(TypeId::of::<T>(), key)
    }
}

impl PartialEq<str> for RecoveryKey {
    fn eq(&self, other: &str) -> bool {
        self.1.eq(other)
    }
}

impl<A: RecoveryMapping> RecoverMapping<A> {
    pub fn create() -> Self {
        let mut resolve = Self { 
            snapshot: ResolveMapper::default(), 
            event: ResolveMapper::default() 
        };
        
        A::mapping(&mut resolve);
        
        resolve
    }
}

impl<A: RecoveryMapping> RecoverMapping<A> {
    pub fn reg_snapshot<S: SnapShot>(&mut self) -> &mut Self
        where A: RecoverSnapShot<S>
    {
        let id = RecoveryKey::new::<S>(S::REGISTRY_KEY);
        if self.snapshot.0.insert(id, Arc::new(SnapShotResolver::<A, S>::default())).is_some() {
            panic!("snapshot: {} was already registered.", S::REGISTRY_KEY)
        }
        self
    }
    
    pub fn reg_event<E: Event>(&mut self) -> &mut Self
        where A: RecoverJournal<E> 
    {
        let id = RecoveryKey::new::<E>(E::REGISTRY_KEY);
        if self.event.0.insert(id, Arc::new(EventResolver::<A, E>::default())).is_some() {
            panic!("event: {} was already registered.", E::REGISTRY_KEY)
        }
        
        self
    }
}


impl<A: RecoveryMapping> RecoverMapping<A> {
    pub fn snapshot(&self) -> &ResolveMapper<A> {
        &self.snapshot
    }
    
    pub fn event(&self) -> &ResolveMapper<A> {
        &self.event
    }

    pub fn is_snapshot_map_empty(&self) -> bool {
        self.snapshot.0.is_empty()
    }
    
    pub fn is_event_map_empty(&self) -> bool {
        self.event.0.is_empty()
    }
}


#[cfg(test)]
mod tests {
    use crate::persistence::mapping::RecoveryKey;

    #[test]
    fn key_id() {
        let id = RecoveryKey::new::<String>("test-key-1");
        println!("{:?}", id);
    }
    
    #[test]
    fn key_id_equals() {
        let a = RecoveryKey::new::<String>("test-key-1");
        let b = RecoveryKey::new::<String>("test-key-1");
        assert_eq!(a, b);
    }
    
    #[test]
    fn key_id_negative() {
        let a = RecoveryKey::new::<u128>("test-key-1");
        let b = RecoveryKey::new::<u128>("test-key-2");
        
        assert_ne!(a, b);

        let a = RecoveryKey::new::<u8>("test-key-1");
        let b = RecoveryKey::new::<String>("test-key-1");
        
        assert_ne!(a, b);
        
        let a = RecoveryKey::new::<i32>("test-key-1");
        let b = RecoveryKey::new::<String>("test-key-2");
        
        assert_ne!(a, b);
    }
}