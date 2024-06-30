use std::any::Any;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::actor::{Actor, Context, Handler, Message, Terminate};
use crate::actor::refs::behavior::{ErrorFlattenBehavior, RegularBehavior};
use crate::errors::ActorError;

pub mod behavior;

pub struct ActorRef<A: Actor> {
    pub(crate) ctx: Arc<RefContext<A>>,
}

#[async_trait::async_trait]
impl<A: Actor> DynRef for ActorRef<A> {
    async fn shutdown(&self) -> Result<(), ActorError> {
        ErrorFlattenBehavior::ask(self, Terminate).await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self {
            ctx: Arc::clone(&self.ctx),
        }
    }
}

pub(crate) struct RefContext<A> {
    pub(crate) sender: UnboundedSender<Box<dyn Applier<A>>>,
}

impl<A: Actor> ActorRef<A> {
    pub(crate) fn new(sender: UnboundedSender<Box<dyn Applier<A>>>) -> ActorRef<A> {
        Self {
            ctx: Arc::new(RefContext { sender }),
        }
    }
}

impl<A: Actor> RegularBehavior<A> for ActorRef<A> {
    async fn ask<M: Message>(
        &self,
        msg: M,
    ) -> Result<Result<A::Accept, A::Rejection>, ActorError>
        where
            A: Handler<M>,
    {
        let (tx, rx) = oneshot::channel();
        let Ok(_) = self.ctx.sender.send(Box::new(Callback {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallBackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallBackSend);
        };

        Ok(res)
    }

    async fn tell<M: Message>(&self, msg: M) -> Result<Result<(), A::Rejection>, ActorError>
        where
            A: Handler<M>,
    {
        let (tx, rx) = oneshot::channel();
        let Ok(_) = self.ctx.sender.send(Box::new(Void {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallBackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallBackSend);
        };

        Ok(res)
    }
}

impl<A: Actor> ErrorFlattenBehavior<A> for ActorRef<A> {
    async fn ask<M: Message>(&self, msg: M) -> Result<A::Accept, A::Rejection>
        where
            A: Handler<M>,
            A::Rejection: From<ActorError>,
    {
        RegularBehavior::ask(self, msg).await.unwrap_or_else(|e| Err(e.into()))
    }

    async fn tell<M: Message>(&self, msg: M) -> Result<(), A::Rejection>
        where
            A: Handler<M>,
            A::Rejection: From<ActorError>,
    {
        RegularBehavior::tell(self, msg).await.unwrap_or_else(|e| Err(e.into()))
    }
}

#[async_trait::async_trait]
pub(crate) trait Applier<A: Actor>: 'static + Sync + Send {
    async fn apply(self: Box<Self>, actor: &mut A, ctx: &mut Context) -> Result<(), ActorError>;
}

pub(crate) struct Callback<A: Actor, M: Message>
where
    A: Handler<M>,
{
    pub(crate) message: M,
    pub(crate) oneshot: oneshot::Sender<Result<A::Accept, A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: Actor, M: Message> Applier<A> for Callback<A, M>
where
    A: Handler<M>,
{
    async fn apply(self: Box<Self>, actor: &mut A, ctx: &mut Context) -> Result<(), ActorError> {
        Ok(self
            .oneshot
            .send(actor.call(self.message, ctx).await)
            .map_err(|_| ActorError::CallBackSend)?)
    }
}

pub(crate) struct Void<A: Actor, M: Message>
where
    A: Handler<M>,
{
    pub(crate) message: M,
    pub(crate) oneshot: oneshot::Sender<Result<(), A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: Actor, M: Message> Applier<A> for Void<A, M>
where
    A: Handler<M>,
{
    async fn apply(self: Box<Self>, actor: &mut A, ctx: &mut Context) -> Result<(), ActorError> {
        match actor.call(self.message, ctx).await {
            Ok(_) => self
                .oneshot
                .send(Ok(()))
                .map_err(|_| ActorError::CallBackSend),
            Err(e) => self
                .oneshot
                .send(Err(e))
                .map_err(|_| ActorError::CallBackSend),
        }
    }
}

#[async_trait::async_trait]
pub trait DynRef: Any {
    async fn shutdown(&self) -> Result<(), ActorError>;
    fn as_any(&self) -> &dyn Any;
}

pub(crate) struct AnyRef(Arc<dyn DynRef + Sync + Send>);

impl AnyRef {
    pub fn downcast<A: Actor>(self) -> Result<ActorRef<A>, ActorError> {
        self
            .0
            .as_any()
            .downcast_ref::<ActorRef<A>>()
            .cloned()
            .ok_or_else(|| ActorError::DownCastFromAny)
    }
}

#[async_trait::async_trait]
impl DynRef for AnyRef {
    async fn shutdown(&self) -> Result<(), ActorError> {
        self.0.shutdown().await
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for AnyRef {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<A: Actor> From<ActorRef<A>> for AnyRef {
    fn from(value: ActorRef<A>) -> Self {
        Self(Arc::new(value))
    }
}
