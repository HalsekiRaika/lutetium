#![cfg(feature = "persistence")]

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use lutetium::actor::{Context, Handler, Message};
use lutetium::actor::refs::{DynRef, RegularAction};
use lutetium::persistence::{PersistenceProvider, RecoverSnapShot, SnapShot, SnapShotProtocol};
use lutetium::persistence::actor::PersistenceActor;
use lutetium::persistence::errors::PersistError;
use lutetium::persistence::identifier::{PersistenceId, ToPersistenceId};
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

impl PersistenceActor for MyActor {
    fn persistence_id(&self) -> PersistenceId {
        self.id.to_persistence_id()
    }
}

#[async_trait::async_trait]
impl Handler<MyCommand> for MyActor {
    type Accept = ();
    type Rejection = MyError;

    async fn call(&mut self, msg: MyCommand, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            MyCommand::Add { k, v } => self.data.insert(k, v),
            MyCommand::Remove { ref k } => self.data.remove(k)
        };
        
        self.snapshot(self, _ctx).await
            .map_err(|_e| MyError::Persist)?;
        
        Ok(())
    }
}

impl SnapShot for MyActor {
    fn as_bytes(&self) -> Result<Vec<u8>, PersistError> {
        flexbuffers::to_vec(self).map_err(|_e| PersistError::Serialization)
    }

    fn from_bytes(bin: &[u8]) -> Result<Self, PersistError> {
        flexbuffers::from_slice(bin).map_err(|_e| PersistError::Serialization)
    }
}

#[async_trait::async_trait]
impl RecoverSnapShot for MyActor {
    async fn recover_snapshot(&mut self, snapshot: MyActor, _ctx: &mut Context) {
        self.data = snapshot.data;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("no context")]
    Persist
}

pub struct InMemoryDataBase {
    db: Arc<RwLock<HashMap<PersistenceId, Vec<u8>>>>
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
impl PersistenceProvider for InMemoryDataBase {
    async fn insert(&self, id: &PersistenceId, bin: Vec<u8>) -> Result<(), PersistError> {
        self.db.write().await.insert(id.to_owned(), bin);
        Ok(())
    }

    async fn select(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError> {
        let bin = self.db.read().await
            .get(id)
            .ok_or(PersistError::NotFound { id: id.to_owned() })?
            .to_owned();
        Ok(bin)
    }

    async fn delete(&self, id: &PersistenceId) -> Result<Vec<u8>, PersistError> {
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