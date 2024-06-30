//! The `behavior` module is used to modify the behavior of [`ActorRef`](crate::actor::refs::ActorRef).
//! 
//! For example, with `persistence` and `event` features enabled, [`PersistenceBehavior`](crate::persistence::event::behavior::PersistenceBehavior) can be used. 
//! Without going into details, this changes the functionality of the [`ActorRef`](crate::actor::refs::ActorRef) to persist the result of an ask/tell.

use std::future::Future;
use crate::actor::{Actor, Handler, Message};
use crate::errors::ActorError;

pub trait RegularBehavior<A: Actor>: 'static + Sync + Send {
    fn ask<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<A::Accept, A::Rejection>, ActorError>> + Send
        where A: Handler<M>;

    fn tell<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<(), A::Rejection>, ActorError>> + Send
        where A: Handler<M>;
}

pub trait ErrorFlattenBehavior<A: Actor>: 'static + Sync + Send {
    fn ask<M: Message>(&self, msg: M) -> impl Future<Output=Result<A::Accept, A::Rejection>> + Send
        where A: Handler<M>,
              A::Rejection: From<ActorError>;

    fn tell<M: Message>(&self, msg: M) -> impl Future<Output=Result<(), A::Rejection>> + Send
        where A: Handler<M>,
              A::Rejection: From<ActorError>;
}