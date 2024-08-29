use crate::actor::{Actor, Context, FromContext};
use crate::errors::ActorError;
use crate::persistence::errors::PersistError;
use crate::persistence::extension::SnapShotProtocol;
use crate::persistence::identifier::PersistenceId;
use crate::persistence::journal::{Event, RecoverJournal};
use crate::persistence::JournalProtocol;
use crate::persistence::recovery::Recovery;
use crate::persistence::snapshot::{RecoverSnapShot, SnapShot};

#[async_trait::async_trait]
pub trait PersistenceActor: 'static + Sync + Send + Sized {
    fn persistence_id(&self) -> PersistenceId;

    #[allow(unused_variables)]
    async fn pre_recovery(&mut self, ctx: &mut Context) -> Result<(), ActorError> { Ok(()) }
    
    #[allow(unused_variables)]
    async fn post_recovery(&mut self, ctx: &mut Context) -> Result<(), ActorError> { Ok(()) }
    
    async fn persist<E: Event>(&self, event: &E, ctx: &mut Context) -> Result<(), PersistError> 
        where Self: RecoverJournal<E>,
    {
        let journal = JournalProtocol::from_context(ctx).await?;
       
        let mut retry = 0;
        while let Err(e) = journal.insert(event).await {
            tracing::error!("{e}");
            retry += 1;
            
            if retry > 5 {
                break;
            }
        }
        
        Ok(())
    }
    
    async fn snapshot<S: SnapShot>(&self, snapshot: &S, ctx: &mut Context) -> Result<(), PersistError> 
        where Self: RecoverSnapShot<S>
    {
        let id = self.persistence_id();
        let store = SnapShotProtocol::from_context(ctx).await?;

        let mut retry = 0;
        while let Err(e) = store.insert(&id, snapshot).await {
            tracing::error!("{}", e);
            retry += 1;
            
            if retry > 5 { 
                break;
            }
        }
        
        Ok(())
    }
    
    async fn recover(&mut self, id: &PersistenceId, ctx: &mut Context) -> Recovery<Self> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor> Actor for A {
    async fn activate(&mut self, ctx: &mut Context) -> Result<(), ActorError> {
        self.pre_recovery(ctx).await?;
        
        // Todo: Impl Auto Recovery Process
        
        self.post_recovery(ctx).await?;
        Ok(())
    }
}

