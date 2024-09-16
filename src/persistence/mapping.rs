use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;
use dashmap::DashMap;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::recovery::{EventResolver, Handler, SnapShotResolver};
use crate::persistence::{Event, RecoverJournal, RecoverSnapShot, SnapShot};

pub trait RecoveryMapping: PersistenceActor {
    fn mapping(mapping: &mut RecoveryMapper<Self>);
}

pub struct RecoveryMapper<A: PersistenceActor> {
    resolve_snapshot: DashMap<TypeId, Arc<dyn Handler<A>>>,
    resolve_events: DashMap<TypeId, Arc<dyn Handler<A>>>,
}

pub struct TypeKeyMapper(DashMap<TypeId, &'static str>);
pub struct ResolveMapper<A: PersistenceActor>(DashMap<String, Arc<dyn Handler<A>>>);

impl<A: RecoveryMapping> RecoveryMapper<A> {
    pub(crate) fn create() -> Self {
        let mut resolve = Self {
            resolve_snapshot: Default::default(),
            resolve_events: Default::default(),
        };
        
        A::mapping(&mut resolve);
        
        resolve
    }
    
    pub fn reg_snapshot<S: SnapShot>(&mut self) -> &mut Self
        where A: RecoverSnapShot<S>
    {
        self.resolve_snapshot.insert(TypeId::of::<S>(), Arc::new(SnapShotResolver::<A, S>::default()));
        self
    }
    
    pub fn reg_event<E: Event>(&mut self) -> &mut Self
        where A: RecoverJournal<E> 
    {
        self.resolve_events.insert(TypeId::of::<E>(), Arc::new(EventResolver::<A, E>::default()));
        self
    }
}