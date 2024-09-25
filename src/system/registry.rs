use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::actor::Actor;
use crate::actor::refs::{ActorRef, AnyRef, DynRef};
use crate::errors::ActorError;
use crate::identifier::ActorId;
use crate::system::{Behavior, LifeCycle};

pub struct Registry(Arc<RwLock<HashMap<ActorId, AnyRef>>>);

impl Registry {
    pub async fn register<A: Actor>(&self, id: ActorId, behavior: Behavior<A>) -> Result<ActorRef<A>, ActorError> {
        if self.0.read().await.contains_key(&id) {
            return Err(ActorError::AlreadySpawned { id })
        }

        let refs = LifeCycle::spawn(behavior).await?;
        
        self.0.write().await
            .insert(id.clone(), AnyRef::from(refs.clone()));
        
        tracing::info!("Registered actor: {}", id);
        
        Ok(refs)
    }

    pub async fn deregister(&self, id: &ActorId) -> Result<(), ActorError> {
        let Some((id, actor)) = self.find(id).await else {
            return Err(ActorError::NotFoundActor { id: id.clone() })
        };
        
        let mut lock = self.0.write().await;
        actor.shutdown().await?;
        lock.remove(&id);
        
        tracing::warn!("De-Registered actor: {}", id);
        Ok(())
    }

    pub async fn find(&self, id: &ActorId) -> Option<(ActorId, AnyRef)> {
        self.0.read().await
            .iter()
            .find(|(dest, _)| dest.eq(&id))
            .map(|(i, a)| (i.clone(), a.clone()))
    }

    pub async fn shutdown_all(&self) -> Result<(), ActorError> {
        for (id, actor) in self.0.read().await.iter() {
            if let Err(e) = actor.shutdown().await {
                tracing::error!("{}: {}", id, e);
            }
        }
        self.0.write().await.clear();
        Ok(())
    }
}

impl Clone for Registry {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
}