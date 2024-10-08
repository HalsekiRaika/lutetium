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
    type Context: ActorContext;
    
    #[allow(unused_variables)]
    async fn activate(&mut self, ctx: &mut Self::Context) -> Result<(), ActorError> {
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

pub trait TryIntoActor<A>: 'static + Sync + Send + Sized {
    type Identifier: IntoActorId;
    type Rejection;
    fn try_into_actor(self, id: Self::Identifier) -> Result<(Self::Identifier, A), Self::Rejection>;
}