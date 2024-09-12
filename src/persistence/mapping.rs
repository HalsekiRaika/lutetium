use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;
use dashmap::DashMap;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::recovery::{EventResolver, Handler, SnapShotResolver};
use crate::persistence::{Event, RecoverJournal, RecoverSnapShot, SnapShot};

pub trait RecoveryMapping: PersistenceActor {
    fn mapping(mapping: &mut RecoverationMapper<Self>);
}

pub struct RecoverationMapper<A: PersistenceActor> {
    resolve_snapshot: DashMap<TypeId, Arc<dyn Handler<A>>>,
    resolve_events: DashMap<TypeId, Arc<dyn Handler<A>>>,
    _mark: PhantomData<A>,
}

impl<A: PersistenceActor> RecoverationMapper<A> {
    pub fn snapshot<S: SnapShot>(&mut self) -> &mut Self 
        where A: RecoverSnapShot<S>
    {
        self.resolve_snapshot.insert(TypeId::of::<S>(), Arc::new(SnapShotResolver::<A, S>::default()));
        self
    }
    
    pub fn event<E: Event>(&mut self) -> &mut Self 
        where A: RecoverJournal<E> 
    {
        self.resolve_events.insert(TypeId::of::<E>(), Arc::new(EventResolver::<A, E>::default()));
        self
    }
}