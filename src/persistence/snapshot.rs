use crate::actor::Context;
use crate::persistence::errors::PersistError;

pub trait SnapShot: 'static + Sync + Send + Sized {
    fn as_byte(&self) -> Result<Vec<u8>, PersistError>;
}

pub trait RecoverSnapShot<S: SnapShot>: 'static + Sync + Send {
    fn recover_snapshot(&mut self, snapshot: S, ctx: &mut Context);
}