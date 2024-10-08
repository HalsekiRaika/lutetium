use crate::actor::Actor;
use crate::identifier::IntoActorId;

pub trait Message: 'static + Sync + Send {}

#[async_trait::async_trait]
pub trait FromMessage<M: Message>: 'static + Sync + Send
    where Self: Actor
{
    type Identifier: IntoActorId;
    type Rejection;
    async fn once(msg: M, ctx: &mut Self::Context) -> Result<(Self::Identifier, Self), Self::Rejection>;
}
