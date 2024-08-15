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
use crate::identifier::IntoActorId;

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

#[async_trait::async_trait]
pub trait Prepare<M: Message>: Actor {
    type Identifier: IntoActorId;
    async fn prepare(msg: M) -> Result<(Self::Identifier, Self), ActorError>;
}