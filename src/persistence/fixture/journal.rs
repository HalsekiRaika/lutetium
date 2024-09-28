use tokio::time::Instant;
use crate::actor::FromContext;
use crate::persistence::SelectionCriteria;
use crate::persistence::actor::PersistenceActor;
use crate::persistence::context::PersistContext;
use crate::persistence::errors::RecoveryError;
use crate::persistence::extension::JournalProtocol;
use crate::persistence::fixture::{Fixable, FixtureParts};
use crate::persistence::identifier::{PersistenceId, SequenceId, Version};
use crate::persistence::mapping::{RecoverMapping, RecoveryMapping};

pub enum Range {
    All,
    StartWith { from: SequenceId }
}

pub struct FixtureJournal<A: PersistenceActor>(Option<Vec<FixtureParts<A>>>);

impl<A: RecoveryMapping> FixtureJournal<A> {
    pub async fn create(id: &PersistenceId, version: &Version, range: Range, ctx: &mut PersistContext) -> Result<Self, RecoveryError> {
        let mapping = RecoverMapping::<A>::create();
        
        if mapping.is_event_map_empty() { 
            tracing::trace!("journal recovery disabled");
            return Ok(Self::disable())
        }
        
        let protocol = JournalProtocol::from_context(ctx).await?;

        let payload = match range {
            Range::All => {
                protocol.read_to_latest(id, version).await?
            }
            Range::StartWith { from } => {
                let select = SelectionCriteria::new(from, SequenceId::max())?;
                protocol.read_to(id, version, select).await?
            }
        };
        
        let Some(journal) = payload else {
            tracing::trace!("journal recovery emptiness");
            return Ok(Self(None));
        };
        
        let fixtures = journal.into_iter()
            .map(|raw| (raw.seq, raw.key, mapping.event().find_by_key(raw.key()), raw.bytes))
            .map(|(seq, key, fixture, bytes)| (seq, fixture.ok_or(RecoveryError::NotCompatible(key)), bytes))
            .map(|(seq, fixture, bytes)| fixture.map(|handler| FixtureParts::new(seq, bytes, handler)))
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
    async fn apply(self, actor: &mut A, ctx: &mut PersistContext) -> Result<(), RecoveryError> {
        let Some(fixtures) = self.0 else {
            return Ok(())
        };

        let now = Instant::now();
        
        for fixture in fixtures {
            fixture.refs.apply(actor, fixture.bytes, ctx).await?;
            ctx.mut_sequence().assign(fixture.seq);
        }
        
        tracing::trace!("events recovered! {}ms", now.elapsed().as_millis());
        
        Ok(())
    }
}