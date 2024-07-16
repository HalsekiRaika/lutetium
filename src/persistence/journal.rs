use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::Context;

pub trait RecoverJournal<M>: 'static + Sync + Send 
    where M: Serialize + DeserializeOwned
{
    fn recover_journal(&mut self, message: M, ctx: &mut Context);
}