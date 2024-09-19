use crate::actor::{Context, FromContext};
use crate::persistence::actor::PersistenceActor;
use crate::persistence::errors::RecoveryError;
use crate::persistence::fixture::FixtureParts;
use crate::persistence::identifier::PersistenceId;
use crate::persistence::JournalProtocol;

pub struct FixtureJournal<A: PersistenceActor>(Option<Vec<FixtureParts<A>>>);

impl<A: PersistenceActor> FixtureJournal<A> {
    pub async fn create(id: &PersistenceId, ctx: &mut Context) -> Result<Self, RecoveryError> {
        let journal = JournalProtocol::from_context(ctx).await?;

        todo!()
    }
}
