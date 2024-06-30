mod handler;
mod message;
mod context;
mod state;
pub mod refs;
mod extension;

pub use self::{
    context::*,
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
