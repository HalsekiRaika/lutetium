#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use lutetium::actor::{Actor, Context, Extension, FromContext, Handler, Message};
use lutetium::actor::refs::{DynRef, RegularAction};
use lutetium::errors::ActorError;
use lutetium::system::ActorSystem;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PersonId(Uuid);

impl Default for PersonId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for PersonId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Person({})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Person {
    id: PersonId,
    name: String,
    age: u32
}

pub enum PersonCommand {
    IncrementAge
}

impl Message for PersonCommand {}

#[derive(Debug, Eq, PartialEq)]
pub enum PersonEvent {
    IncrementedAge
}

impl Actor for Person { type Context = Context; }

#[async_trait]
impl Handler<PersonCommand> for Person {
    type Accept = PersonEvent;
    type Rejection = ActorError;

    #[tracing::instrument(skip_all, name = "Person")]
    async fn call(&mut self, msg: PersonCommand, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            PersonCommand::IncrementAge => {
                let Extension(s): Extension<String> = Extension::from_context(ctx).await?;
                tracing::debug!(name: "extension", "{s}");

                self.age += 1;

                Ok(PersonEvent::IncrementedAge)
            },
        }
    }
}

#[tokio::test]
async fn system_build() -> anyhow::Result<()> {
    let test = "extension".to_string();
    
    let mut system = ActorSystem::builder();
    
    system.extension(move |ext| {
        ext.install(test);
    });
    
    let _system = system.build();
    
    Ok(())
}

#[tokio::test]
async fn extract_extension_in_handler() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();

    let test = "extension".to_string();

    let mut system = ActorSystem::builder();

    system.extension(move |ext| {
        ext.install(test);
    });

    let system = system.build();

    let id = PersonId::default();
    let person = Person {
        id,
        name: "RechellaTek".to_string(),
        age: 21,
    };

    let refs = system.spawn(id, person).await?;

    refs.tell(PersonCommand::IncrementAge).await??;
    refs.shutdown().await?;

    Ok(())
}