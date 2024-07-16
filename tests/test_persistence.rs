#![cfg(feature = "persistence")]

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use lutetium::actor::{Context, Handler, Message};
use lutetium::actor::refs::RegularAction;
use lutetium::errors::ActorError;
use lutetium::persistence::actor::PersistenceActor;
use lutetium::persistence::{SnapShotProtocol, PersistenceProvider};
use lutetium::persistence::errors::PersistError;
use lutetium::persistence::identifier::PersistenceId;
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
    
}

#[async_trait::async_trait]
impl Handler<MyCommand> for MyActor {
    type Accept = ();
    type Rejection = ActorError;

    async fn call(&mut self, msg: MyCommand, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            MyCommand::Add { k, v } => self.data.insert(k, v),
            MyCommand::Remove { ref k } => self.data.remove(k)
        };
        
        Ok(())
    }
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
    
    Ok(())
}