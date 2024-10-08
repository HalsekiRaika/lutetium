#![allow(unused)]

use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use lutetium::actor::{Actor, ActorContext, Context, Handler, Message};
use lutetium::actor::refs::{DynRef, RegularAction};
use lutetium::errors::ActorError;
use lutetium::system::{ActorSystem, LutetiumActorSystem};

#[derive(Debug, Copy, Clone)]
pub struct State {
    id: Uuid,
    state: i32
}

impl Actor for State { type Context = Context; }

pub struct Command;

impl Message for Command {}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error(transparent)]
    ActorSpawn(ActorError)
}

#[async_trait::async_trait]
impl Handler<Command> for State {
    type Accept = ();
    type Rejection = ErrorKind;

    async fn call(&mut self, msg: Command, ctx: &mut Self::Context) -> Result<Self::Accept, Self::Rejection> {
        ctx.shutdown().await;
        Ok(())
    }
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
            .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    
    let system = ActorSystem::builder().build();
    
    for state in 0..50 {
        let id = Uuid::now_v7();
        let state = State { id, state };
        system.spawn(id, state).await?;
    }
    
    system.shutdown_all().await?;
    
    Ok(())
}

#[tokio::test]
async fn self_shutdown() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();

    let system = ActorSystem::builder().build();

    let id = Uuid::now_v7();
    let refs = system.spawn(id, State { id, state: 5 }).await?;
    refs.tell(Command).await??;
    
    Ok(())
}

#[tokio::test]
async fn refs_shutdown() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();

    let system = ActorSystem::builder().build();

    let id = Uuid::now_v7();
    let refs = system.spawn(id, State { id, state: 5 }).await?;

    refs.shutdown().await?;
    Ok(())
}