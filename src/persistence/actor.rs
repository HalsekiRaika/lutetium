use crate::actor::{Actor, Context, FromContext};
use crate::errors::ActorError;
use crate::persistence::JournalProtocol;
use crate::persistence::errors::{PersistError, RecoveryError};
use crate::persistence::extension::SnapShotProtocol;
use crate::persistence::fixture::{Fixable, Fixture, FixtureJournal, FixtureSnapShot, Range};
use crate::persistence::identifier::PersistenceId;
use crate::persistence::journal::{Event, RecoverJournal};
use crate::persistence::mapping::RecoveryMapping;
use crate::persistence::snapshot::{RecoverSnapShot, SnapShot};

#[async_trait::async_trait]
pub trait PersistenceActor: 'static + Sync + Send + Sized {
    fn persistence_id(&self) -> PersistenceId;
    
    #[allow(unused_variables)]
    async fn pre_recovery(&mut self, ctx: &mut Context) -> Result<(), ActorError> { Ok(()) }
    
    #[allow(unused_variables)]
    async fn post_recovery(&mut self, ctx: &mut Context) -> Result<(), ActorError> { Ok(()) }
    
    async fn persist<E: Event>(&self, event: &E, ctx: &mut Context) -> Result<(), PersistError> 
        where Self: RecoverJournal<E> + RecoveryMapping,
    {
        let id = self.persistence_id();
        let journal = JournalProtocol::from_context(ctx).await?;
       
        let mut retry = 0;
        while let Err(e) = journal.write_to_latest(&id, event).await {
            tracing::error!("{e}");
            retry += 1;
            
            if retry > 5 {
                break;
            }
        }
        
        Ok(())
    }
    
    async fn snapshot<S: SnapShot>(&self, snapshot: &S, ctx: &mut Context) -> Result<(), PersistError> 
        where Self: RecoverSnapShot<S> + RecoveryMapping
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
    
    async fn recover(&mut self, id: &PersistenceId, ctx: &mut Context) -> Result<Fixture<Self>, RecoveryError> 
        where Self: RecoveryMapping
    {
        // Todo: To store Journal sequence values in Snapshot to reduce extra loading.
        let sf = FixtureSnapShot::create(id, ctx).await?;
        let jf = FixtureJournal::create(id, Range::All, ctx).await?;
        Ok(Fixture::new(sf, jf))
    }
}

#[async_trait::async_trait]
impl<A: RecoveryMapping> Actor for A {
    async fn activate(&mut self, ctx: &mut Context) -> Result<(), ActorError> {
        self.pre_recovery(ctx).await?;

        let id = self.persistence_id();
        
        let fixture = self.recover(&id, ctx).await
            .map_err(|e| ActorError::External(Box::new(e)))?;
        
        fixture.apply(self, ctx).await
            .map_err(|e| ActorError::External(Box::new(e)))?;
        
        self.post_recovery(ctx).await?;
        Ok(())
    }
}

