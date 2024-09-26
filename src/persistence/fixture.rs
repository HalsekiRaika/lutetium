mod journal;
mod snapshot;

pub use self::{
    journal::*,
    snapshot::*,
};

use std::sync::Arc;

use crate::persistence::actor::PersistenceActor;
use crate::persistence::context::PersistContext;
use crate::persistence::errors::RecoveryError;
use crate::persistence::identifier::SequenceId;
use crate::persistence::mapping::RecoveryMapping;
use crate::persistence::recovery::Handler;

pub struct Fixture<A: PersistenceActor> {
    snapshot: FixtureSnapShot<A>,
    journal: FixtureJournal<A>, 
}

impl<A: RecoveryMapping> Fixture<A> {
    pub fn new(snapshot: FixtureSnapShot<A>, journal: FixtureJournal<A>) -> Self {
        Self { snapshot, journal }
    }
}

#[async_trait::async_trait]
impl<A: PersistenceActor> Fixable<A> for Fixture<A> {
    async fn apply(self, actor: &mut A, ctx: &mut PersistContext) -> Result<(), RecoveryError> {
        self.snapshot.apply(actor, ctx).await?;
        self.journal.apply(actor, ctx).await?;
        Ok(())
    }
}

pub struct FixtureParts<A: PersistenceActor> {
    pub(crate) seq: SequenceId,
    pub(crate) bytes: Vec<u8>,
    pub(crate) refs: Arc<dyn Handler<A>>
}

impl<A: PersistenceActor> FixtureParts<A> {
    pub fn new(seq: SequenceId, bytes: Vec<u8>, refs: Arc<dyn Handler<A>>) -> Self {
        Self { seq, bytes, refs }
    }
}

#[async_trait::async_trait]
pub trait Fixable<A: PersistenceActor>: 'static + Sync + Send {
    async fn apply(self, actor: &mut A, ctx: &mut PersistContext) -> Result<(), RecoveryError>;
}