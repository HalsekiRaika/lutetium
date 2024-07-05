use crate::actor::{Actor, Context, Handler, Message};
use crate::errors::ActorError;
use self::private::Sealed;

#[async_trait::async_trait]
pub trait SealedHandler<M: Message>: 'static + Sync + Send
    where Self: Sealed + Actor
{
    type Accept: Sync + Send + 'static;
    type Rejection: Sync + Send + 'static;
    async fn call(&mut self, msg: M, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection>;
}

#[async_trait::async_trait]
impl<Sealed: SealedHandler<M>, M: Message> Handler<M> for Sealed {
    type Accept = Sealed::Accept;
    type Rejection = Sealed::Rejection;
    async fn call(&mut self, msg: M, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        self.call(msg, ctx).await
    }
}



#[derive(Eq, PartialEq)]
pub struct Terminate;

impl Message for Terminate {}

#[async_trait::async_trait]
impl<A: Actor + Sealed> SealedHandler<Terminate> for A {
    type Accept = ();
    type Rejection = ActorError;

    async fn call(&mut self, _: Terminate, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        tracing::warn!("received terminate signal.");
        ctx.shutdown();
        Ok(())
    }
}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for T {}
}