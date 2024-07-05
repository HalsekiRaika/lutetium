//! The `action` module is used to modify the conduct of [`ActorRef`](crate::actor::refs::ActorRef).

use std::future::Future;
use crate::actor::{Actor, Handler, Message};
use crate::errors::ActorError;

pub trait RegularAction<A: Actor>: 'static + Sync + Send {
    fn ask<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<A::Accept, A::Rejection>, ActorError>> + Send
        where A: Handler<M>;

    fn tell<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<(), A::Rejection>, ActorError>> + Send
        where A: Handler<M>;
}

pub trait ErrorFlattenAction<A: Actor>: 'static + Sync + Send {
    fn ask<M: Message>(&self, msg: M) -> impl Future<Output=Result<A::Accept, A::Rejection>> + Send
        where A: Handler<M>,
              A::Rejection: From<ActorError>;

    fn tell<M: Message>(&self, msg: M) -> impl Future<Output=Result<(), A::Rejection>> + Send
        where A: Handler<M>,
              A::Rejection: From<ActorError>;
}