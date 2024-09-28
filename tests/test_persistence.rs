#![cfg(feature = "persistence")]

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use lutetium::actor::{Handler, Message, TryIntoActor};
use lutetium::actor::refs::{DynRef, RegularAction};
use lutetium::persistence::{Event, RecoverJournal, RecoverSnapShot, SnapShot, SelectionCriteria, PersistContext};
use lutetium::persistence::actor::PersistenceActor;
use lutetium::persistence::errors::{DeserializeError, PersistError, SerializeError};
use lutetium::persistence::extension::{JournalPayload, JournalProtocol, JournalProvider, SnapShotPayload, SnapShotProtocol, SnapShotProvider};
use lutetium::persistence::identifier::{PersistenceId, SequenceId, ToPersistenceId, Version};
use lutetium::persistence::mapping::{RecoverMapping, RecoveryMapping};
use lutetium::system::ActorSystem;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyActor {
    id: Uuid,
    data: HashMap<String, String>
}

#[derive(Debug, Clone)]
pub enum MyCommand {
    Create,
    Add { k: String, v: String },
    Remove { k: String }
}

impl TryIntoActor for MyCommand {
    type Identifier = Uuid;
    type Actor = MyActor;
    type Rejection = anyhow::Error;

    fn try_into_actor(self, id: Self::Identifier) -> Result<(Self::Identifier, Self::Actor), Self::Rejection> {
        Ok((id, MyActor { id, data: Default::default() }))
    }
}

impl Message for MyCommand {
    
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MyEvent {
    Created { id: Uuid },
    Added { k: String, v: String },
    Removed { k: String }
}

impl Event for MyEvent {
    const REGISTRY_KEY: &'static str = "my-actor-event";


    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(flexbuffers::to_vec(self)?)
    }
    
    fn from_bytes(bin: &[u8]) -> Result<Self, DeserializeError> {
        Ok(flexbuffers::from_slice(bin)?)
    }
}

impl SnapShot for MyActor {
    const REGISTRY_KEY: &'static str = "my-actor-snapshot";


    fn as_bytes(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(flexbuffers::to_vec(self)?)
    }
    
    fn from_bytes(bin: &[u8]) -> Result<Self, DeserializeError> {
        Ok(flexbuffers::from_slice(bin)?)
    }
}

#[async_trait::async_trait]
impl RecoverSnapShot for MyActor {
    async fn recover_snapshot(&mut self, snapshot: MyActor, _ctx: &mut PersistContext) {
        tracing::trace!("recovered snapshot: {:?}", snapshot);
        self.data = snapshot.data;
    }
}

#[async_trait::async_trait]
impl RecoverJournal<MyEvent> for MyActor {
    async fn recover_journal(&mut self, event: MyEvent, _ctx: &mut PersistContext) {
        tracing::trace!("recovered event: {:?}", event);
        match event {
            MyEvent::Created { id } => {
                self.id = id;
            }
            MyEvent::Added { k, v } => {
                self.data.insert(k, v);
            }
            MyEvent::Removed { ref k } => {
                self.data.remove(k);
            }
        }
    }
}

impl PersistenceActor for MyActor {
    const VERSION: Version = Version::new("0.0.1");
    
    fn persistence_id(&self) -> PersistenceId {
        self.id.to_persistence_id()
    }
}

impl RecoveryMapping for MyActor {
    fn mapping(mapping: &mut RecoverMapping<Self>) {
        mapping
            .reg_snapshot::<Self>()
            .reg_event::<MyEvent>();
    }
}

#[async_trait::async_trait]
impl Handler<MyCommand> for MyActor {
    type Accept = MyEvent;
    type Rejection = MyError;

    async fn call(&mut self, msg: MyCommand, ctx: &mut PersistContext) -> Result<Self::Accept, Self::Rejection> {
        let ev = match msg {
            MyCommand::Create => {
                self.snapshot(self, ctx).await
                    .map_err(|e| MyError::Persist(anyhow::Error::new(e)))?;
                
                MyEvent::Created { id: self.id }
            }
            MyCommand::Add { k, v } => {
                self.data.insert(k.clone(), v.clone());
                MyEvent::Added { k, v }
            }
            MyCommand::Remove { k } => {
                self.data.remove(&k);
                MyEvent::Removed { k }
            }
        };
        
        self.persist(&ev, ctx).await
            .map_err(|e| MyError::Persist(anyhow::Error::new(e)))?;
        
        Ok(ev)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error(transparent)]
    Persist(anyhow::Error)
}

#[allow(clippy::type_complexity)]
pub struct InMemorySnapShotStore {
    db: Arc<RwLock<HashMap<Version, HashMap<(PersistenceId, SequenceId), SnapShotPayload>>>>
}

impl Clone for InMemorySnapShotStore {
    fn clone(&self) -> Self {
        Self { db: Arc::clone(&self.db) }
    }
}

impl Default for InMemorySnapShotStore {
    fn default() -> Self {
        Self { db: Arc::new(RwLock::new(HashMap::default())) }
    }
}

#[async_trait::async_trait]
impl SnapShotProvider for InMemorySnapShotStore {
    async fn insert(&self, id: &PersistenceId, version: &Version, seq: &SequenceId, payload: SnapShotPayload) -> Result<(), PersistError> {
        let mut lock = self.db.write().await;
        if let Some((_, store)) = lock.iter_mut().find(|(ver, _)| ver.eq(&version)) {
            store.insert((id.to_owned(), seq.to_owned()), payload);
        } else {
            let mut store = HashMap::new();
            store.insert((id.to_owned(), seq.to_owned()), payload);
            
            lock.insert(version.to_owned(), store);
        }
        
        Ok(())
    }

    async fn select(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<SnapShotPayload>, PersistError> {
        let bin = self.db.read().await
            .iter()
            .find(|(ver, _)| ver.eq(&version))
            .and_then(|(_, store)| {
                store.iter()
                    .find(|((pid, sq), _)| pid.eq(id) && sq <= seq)
                    .map(|(_, payload)| payload)
                    .cloned()
            });
        Ok(bin)
    }
}

#[allow(clippy::type_complexity)]
pub struct InMemoryJournalStore {
    db: Arc<RwLock<HashMap<Version, HashMap<(PersistenceId, SequenceId), JournalPayload>>>>
}

impl Clone for InMemoryJournalStore {
    fn clone(&self) -> Self {
        Self { db: Arc::clone(&self.db) }
    }
}

impl Default for InMemoryJournalStore {
    fn default() -> Self {
        Self { db: Arc::new(RwLock::new(HashMap::new())) }
    }
}

#[async_trait::async_trait]
impl JournalProvider for InMemoryJournalStore {
    async fn insert(&self, id: &PersistenceId, version: &Version, seq: &SequenceId, msg: JournalPayload) -> Result<(), PersistError> {
        let mut lock = self.db.write().await;
        if let Some((_, store)) = lock.iter_mut().find(|(ver, _)| ver.eq(&version)) {
            store.insert((id.to_owned(), seq.to_owned()), msg);
        } else {
            let mut store = HashMap::new();
            store.insert((id.to_owned(), seq.to_owned()), msg);
            
            lock.insert(version.to_owned(), store);
        }
        
        Ok(())
    }

    async fn select_one(&self, id: &PersistenceId, version: &Version, seq: &SequenceId) -> Result<Option<JournalPayload>, PersistError> {
        let payload = self.db.read().await
            .iter()
            .find(|(ver, _)| ver.eq(&version))
            .and_then(|(_, store)| {
                store.iter()
                    .find(|((pid, sq), _)| pid.eq(id) && sq.eq(seq))
                    .map(|(_, payload)| payload)
                    .cloned()
            });
        
        Ok(payload)
    }

    async fn select_many(&self, id: &PersistenceId, version: &Version, criteria: SelectionCriteria) -> Result<Option<BTreeSet<JournalPayload>>, PersistError> {
        let col = self.db.read().await
            .iter()
            .find(|(ver, _)| ver.eq(&version))
            .map(|(_, store)| {
                store.iter()
                    .filter(|((pid, sq), _)| pid.eq(id) && criteria.matches(sq))
                    .map(|(_, payload)| payload)
                    .cloned()
                    .collect::<BTreeSet<_>>()
            });
        
        Ok(col)
    }
}


#[tokio::test]
async fn persistence_actor_run() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    
    let mut system = ActorSystem::builder();
    
    system.extension(|ext| {
        ext.install(SnapShotProtocol::new(InMemorySnapShotStore::default()));
        ext.install(JournalProtocol::new(InMemoryJournalStore::default()));
    });
    
    let system = system.build();
    
    let id = Uuid::now_v7();
    
    let create = MyCommand::Create;
    
    let refs = system.try_spawn(id, create.clone()).await??;
    
    refs.tell(create).await??; 
    refs.tell(MyCommand::Add { k: "aaa".to_string(), v: "111".to_string() }).await??;
    refs.tell(MyCommand::Add { k: "bbb".to_string(), v: "222".to_string() }).await??;
    refs.tell(MyCommand::Add { k: "ccc".to_string(), v: "333".to_string() }).await??;
    refs.tell(MyCommand::Add { k: "ddd".to_string(), v: "444".to_string() }).await??;
    refs.tell(MyCommand::Add { k: "eee".to_string(), v: "555".to_string() }).await??;
    refs.tell(MyCommand::Add { k: "fff".to_string(), v: "666".to_string() }).await??;
    refs.tell(MyCommand::Remove { k: "bbb".to_string() }).await??;
    refs.tell(MyCommand::Remove { k: "ddd".to_string() }).await??;
    refs.tell(MyCommand::Remove { k: "fff".to_string() }).await??;
    
    system.shutdown(&id).await?;
    
    let refs = system.try_spawn(id, MyCommand::Create).await??;
    
    refs.shutdown().await?;
    
    Ok(())
}