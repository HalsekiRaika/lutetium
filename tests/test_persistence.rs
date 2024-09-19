#![cfg(feature = "persistence")]

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use lutetium::actor::{Context, Handler, Message};
use lutetium::actor::refs::{DynRef, RegularAction};
use lutetium::persistence::{Event, SnapShotProvider, RecoverJournal, RecoverSnapShot, SnapShot, SnapShotProtocol, SnapShotPayload};
use lutetium::persistence::actor::PersistenceActor;
use lutetium::persistence::errors::{DeserializeError, PersistError, SerializeError};
use lutetium::persistence::identifier::{PersistenceId, ToPersistenceId};
use lutetium::persistence::mapping::{RecoverMapping, RecoveryMapping};
use lutetium::system::ActorSystem;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyActor {
    id: Uuid,
    data: HashMap<String, String>
}

pub enum MyCommand {
    Add { k: String, v: String },
    Remove { k: String }
}

impl Message for MyCommand {
    
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MyEvent {
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
    async fn recover_snapshot(&mut self, snapshot: MyActor, _ctx: &mut Context) {
        self.data = snapshot.data;
    }
}

#[async_trait::async_trait]
impl RecoverJournal<MyEvent> for MyActor {
    async fn recover_journal(&mut self, event: MyEvent, _ctx: &mut Context) {
        match event {
            MyEvent::Added { k, v } => {
                self.data.insert(k, v);
            },
            MyEvent::Removed { ref k } => {
                self.data.remove(k);
            },
        }
    }
}

impl PersistenceActor for MyActor {
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

    async fn call(&mut self, msg: MyCommand, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        let ev = match msg {
            MyCommand::Add { k, v } => {
                self.data.insert(k.clone(), v.clone());
                MyEvent::Added { k, v }
            },
            MyCommand::Remove { k } => {
                self.data.remove(&k);
                MyEvent::Removed { k }
            }
        };
        
        self.snapshot(self, _ctx).await
            .map_err(|e| MyError::Persist(anyhow::Error::new(e)))?;
        
        // self.persist(&ev, _ctx).await
        //     .map_err(|e| MyError::Persist(anyhow::Error::new(e)))?;
        
        Ok(ev)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error(transparent)]
    Persist(anyhow::Error)
}

pub struct InMemoryDataBase {
    db: Arc<RwLock<HashMap<PersistenceId, SnapShotPayload>>>
}

impl Clone for InMemoryDataBase {
    fn clone(&self) -> Self {
        Self { db: Arc::clone(&self.db) }
    }
}

impl Default for InMemoryDataBase {
    fn default() -> Self {
        Self { db: Arc::new(RwLock::new(HashMap::default())) }
    }
}

#[async_trait::async_trait]
impl SnapShotProvider for InMemoryDataBase {
    async fn insert(&self, id: &PersistenceId, payload: SnapShotPayload) -> Result<(), PersistError> {
        self.db.write().await.insert(id.to_owned(), payload);
        Ok(())
    }

    async fn select(&self, id: &PersistenceId) -> Result<Option<SnapShotPayload>, PersistError> {
        let bin = self.db.read().await
            .get(id)
            .cloned();
        Ok(bin)
    }

    async fn delete(&self, id: &PersistenceId) -> Result<SnapShotPayload, PersistError> {
        let bin = self.db.write().await
            .remove(id)
            .ok_or(PersistError::NotFound { id: id.to_owned() })?;
        Ok(bin)
    }
}

#[tokio::test]
async fn persistence_actor_run() -> anyhow::Result<()> {
    let mut system = ActorSystem::builder();
    
    system.extension(|ext| {
        ext.install(SnapShotProtocol::new(InMemoryDataBase::default()));
    });
    
    let system = system.build();
    
    let id = Uuid::now_v7();
    let actor = MyActor { id, data: HashMap::new() };
    
    let refs = system.spawn(id, actor).await?;
    
    refs.tell(MyCommand::Add { k: "aaa".to_string(), v: "111".to_string() }).await??;
    refs.shutdown().await?;
    
    Ok(())
}