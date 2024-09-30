#![allow(unused)]

use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use lutetium::actor::{Actor, Context};
use lutetium::system::ActorSystem;

#[derive(Debug, Copy, Clone)]
pub struct State {
    id: Uuid,
    state: i32
}

impl Actor for State { type Context = Context; }

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