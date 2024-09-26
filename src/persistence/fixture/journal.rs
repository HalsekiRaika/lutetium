use tokio::time::Instant;
use crate::actor::{Context, FromContext};
use crate::persistence::{JournalProtocol, SelectionCriteria};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::errors::RecoveryError;
use crate::persistence::fixture::{Fixable, FixtureParts};
use crate::persistence::identifier::{PersistenceId, SequenceId};
use crate::persistence::mapping::{RecoverMapping, RecoveryMapping};

pub enum Range {
    All,
    StartWith { from: SequenceId }
}

pub struct FixtureJournal<A: PersistenceActor>(Option<Vec<FixtureParts<A>>>);

impl<A: RecoveryMapping> FixtureJournal<A> {
    pub async fn create(id: &PersistenceId, range: Range, ctx: &mut Context) -> Result<Self, RecoveryError> {
        let mapping = RecoverMapping::<A>::create();
        
        if mapping.is_event_map_empty() { 
            tracing::trace!("journal recovery disabled");
            return Ok(Self::disable())
        }
        
        let journal = JournalProtocol::from_context(ctx).await?;

        let payload = match range {
            Range::All => {
                journal.read_to_latest(id).await?
            }
            Range::StartWith { from } => {
                let select = SelectionCriteria::new(from, SequenceId::max())?;
                journal.read_to(id, select).await?
            }
        };
        
        if payload.is_empty() {
            tracing::trace!("journal recovery emptiness");
            return Ok(Self(None));
        }
        
        let fixtures = payload.into_iter()
            .map(|raw| (raw.key(), mapping.event().find(raw.key()), raw.bytes))
            .map(|(key, fixture, bytes)| (fixture.ok_or(RecoveryError::NotCompatible(key)), bytes))
            .map(|(fixture, bytes)| fixture.map(|handler| FixtureParts::new(bytes, handler)))
            .collect::<Result<Vec<FixtureParts<A>>, _>>()?;
        
        
        Ok(Self(Some(fixtures)))
    }
    
    pub fn disable() -> Self {
        Self(None)
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor> Fixable<A> for FixtureJournal<A> {
    async fn apply(self, actor: &mut A, ctx: &mut Context) -> Result<(), RecoveryError> {
        let Some(fixtures) = self.0 else {
            return Ok(())
        };

        let now = Instant::now();
        
        for fixture in fixtures {
            fixture.refs.apply(actor, fixture.bytes, ctx).await?
        }
        
        tracing::trace!("events recovered! {}ms", now.elapsed().as_millis());
        
        Ok(())
    }
}