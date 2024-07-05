mod extension;
mod handler;
mod message;
mod context;
mod state;
pub mod refs;

pub use self::{
    context::*,
    extension::*,
    handler::*,
    message::*,
    state::*,
};

use crate::errors::ActorError;

#[async_trait::async_trait]
pub trait Actor: 'static + Sync + Send + Sized {
    async fn activate(&mut self, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::debug!(name: "actor", "activate");
        Ok(())
    }
}
