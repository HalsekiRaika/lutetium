#![allow(unused)]
use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use lutetium::actor::{Actor, Context, Handler, Message, TryIntoActor};
use lutetium::actor::refs::{ActorRef, DynRef, ErrorFlattenAction};
use lutetium::errors::ActorError;
use lutetium::identifier::IntoActorId;
use lutetium::system::{ActorSystem, LutetiumActorSystem};

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
    Spawn,
    IncrementAge
}

impl Message for PersonCommand {}

#[derive(Debug, Eq, PartialEq)]
pub enum PersonEvent {
    IncrementedAge
}

impl Actor for Person { type Context = Context; }

impl TryIntoActor<Person> for PersonCommand {
    type Identifier = PersonId;
    type Rejection = anyhow::Error;

    fn try_into_actor(self, id: Self::Identifier) -> Result<(Self::Identifier, Person), Self::Rejection> {
        let person = Person { id, name: "into man".to_string(), age: 0, };
        Ok((id, person))
    }
}

#[async_trait]
impl Handler<PersonCommand> for Person {
    type Accept = PersonEvent;
    type Rejection = ActorError;
    
    #[tracing::instrument(skip_all, name = "Person")]
    async fn call(&mut self, msg: PersonCommand, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            PersonCommand::IncrementAge => {
                self.age += 1;
                tracing::debug!("person was old {}", self.age);
                Ok(PersonEvent::IncrementedAge)
            },
            _ => unreachable!()
        }
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
    
    let id = PersonId::default();
    let person = Person {
        id,
        name: "RechellaTek".to_string(),
        age: 21,
    };
    
    let refs = system.spawn(id, person.clone()).await?;
    
    let event = refs.ask(PersonCommand::IncrementAge).await?;
    
    assert_eq!(event, PersonEvent::IncrementedAge);
    
    system.shutdown(&id).await?;
    
    let refs = system.find_or(id, |_id| async move {
        person
    }).await;
    
    Ok(())
}

#[tokio::test]
async fn prepare_spawn() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_filter(tracing_subscriber::EnvFilter::new("test=trace,lutetium=trace"))
            .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    
    let system = ActorSystem::builder().build();
    
    let id = PersonId::default();
    let refs: ActorRef<Person> = system.try_spawn(id, PersonCommand::Spawn).await??;
    
    let event = refs.ask(PersonCommand::IncrementAge).await?;
    
    assert_eq!(event, PersonEvent::IncrementedAge);
    
    refs.shutdown().await?;
    
    Ok(())
}