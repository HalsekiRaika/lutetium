use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::actor::Actor;
use crate::actor::refs::{ActorRef, AnyRef, DynRef};
use crate::errors::ActorError;
use crate::identifier::ActorId;
use crate::system::Behavior;
use crate::system::lifecycle::LifeCycle;

pub(crate) struct Registry(Arc<RwLock<HashMap<ActorId, AnyRef>>>);

impl Registry {
    pub async fn register<A: Actor>(&self, id: ActorId, behavior: Behavior<A>) -> Result<ActorRef<A>, ActorError> {
        if let Some((_, actor)) = self.find(&id).await { 
            if actor.is_active().await {
                return Err(ActorError::AlreadySpawned { id })
            }
        }
        
        let refs = LifeCycle::spawn(self.clone(), behavior).await?;

        if self.0.write().await
            .insert(id.clone(), AnyRef::from(refs.clone())).is_some()
        {
            tracing::warn!("Actor during shutdown in the registry has been overwritten.");
        }
        
        tracing::info!("Registered actor: {}", id);
        
        Ok(refs)
    }

    /// Deregister Actor from the [`Registry`].
    /// 
    /// At first glance, it looks like just sending a shutdown signal to the Actor, 
    /// but in the [`lifecycle`](crate::system::lifecycle), [`Registry::untracked`] is called.
    /// 
    /// The sequence is as follows
    /// 
    /// 1. Shutdown signal is sent to Actor
    /// 2. Lifecycle checks shutdown flag
    /// 3. Channel is broken
    /// 4. Registry is told to remove itself from tracking
    /// 5. Complete
    pub async fn deregister(&self, id: &ActorId) -> Result<(), ActorError> {
        let Some((id, actor)) = self.find(id).await else {
            return Err(ActorError::NotFoundActor { id: id.clone() })
        };
        
        actor.shutdown().await?;
        
        tracing::warn!("De-Registered actor: {}", id);
        Ok(())
    }
    
    #[allow(unused)]
    pub async fn track<A: Actor>(&self, id: ActorId, refs: ActorRef<A>) -> Result<(), ActorError> {
        if (self.find(&id).await).is_some() {
            return Err(ActorError::AlreadySpawned { id })
        }
        
        self.0.write().await
            .insert(id.clone(), AnyRef::from(refs.clone()));
        
        Ok(())
    }
    
    /// Remove the `ActorRef` indicated by Identifier from the current registry tracking.
    /// 
    /// This will cause the ActorRef to no longer be maintained, 
    /// so the Actor will stop as soon as all known ActorRefs of the target are dropped.
    pub async fn untracked(&self, id: &ActorId) -> Result<(), ActorError> {
        let Some((id, _)) = self.find(id).await else {
            return Err(ActorError::NotFoundActor { id: id.clone() })
        };
        
        let mut lock = self.0.write().await;
        if lock.remove(&id).is_none() {
            return Err(ActorError::NotFoundActor { id: id.to_owned() })
        }
        
        tracing::warn!("untracked actor: {}", id);
        Ok(())
    }

    pub async fn find(&self, id: &ActorId) -> Option<(ActorId, AnyRef)> {
        self.0.read().await
            .iter()
            .find(|(dest, _)| dest.eq(&id))
            .map(|(i, a)| (i.clone(), a.clone()))
    }

    pub async fn shutdown_all(&self) -> Result<(), ActorError> {
        let lock = self.0.read().await;
        for (id, actor) in lock.iter() {
            if let Err(e) = actor.shutdown().await {
                tracing::error!("{}: {}", id, e);
            }
        }
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