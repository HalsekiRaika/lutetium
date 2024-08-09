pub mod refs;
mod extension;
mod handler;
mod message;
mod context;
mod state;

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

impl<A: Actor> IntoActor for A {
    type Actor = Self;
    fn into_actor(self) -> Self::Actor {
        self
    }
}


pub trait IntoActor: 'static + Sync + Send + Sized {
    type Actor: Actor;
    fn into_actor(self) -> Self::Actor;
}