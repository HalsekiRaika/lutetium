mod sealed;

use crate::actor::{Actor, Context, Message};

pub use self::sealed::*;

#[async_trait::async_trait]
pub trait Handler<M: Message>: 'static + Sync + Send
where
    Self: Actor,
{
    type Accept: 'static + Sync + Send;
    type Rejection: 'static + Sync + Send;
    async fn call(
        &mut self,
        msg: M,
        ctx: &mut Context
    ) -> Result<Self::Accept, Self::Rejection>;
}