#![allow(unused)]
use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use lutetium::actor::{Actor, Context, Handler, Message, Prepare};
use lutetium::actor::refs::{ActorRef, DynRef, ErrorFlattenAction};
use lutetium::errors::ActorError;
use lutetium::identifier::IntoActorId;
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
    Spawn,
    IncrementAge
}

impl Message for PersonCommand {}

#[derive(Debug, Eq, PartialEq)]
pub enum PersonEvent {
    IncrementedAge
}

impl Actor for Person {}

#[async_trait]
impl Prepare<PersonCommand> for Person {
    type Identifier = PersonId;
    async fn prepare(msg: PersonCommand) -> Result<(Self::Identifier, Self), ActorError> {
        if let PersonCommand::Spawn = msg {
            let id = PersonId::default();
            let p = Person {
                id,
                name: "RechellaTek".to_string(),
                age: 21,
            };
            return Ok((id, p))
        };
        Err(ActorError::NotEnoughValue)
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
    
    let refs = system.spawn(id, person).await?;
    
    let event = refs.ask(PersonCommand::IncrementAge).await?;
    
    assert_eq!(event, PersonEvent::IncrementedAge);
    
    system.shutdown(id).await?;
    
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
    
    let refs: ActorRef<Person> = system.spawn_from(PersonCommand::Spawn).await?;
    
    let event = refs.ask(PersonCommand::IncrementAge).await?;
    
    assert_eq!(event, PersonEvent::IncrementedAge);
    
    refs.shutdown().await?;
    
    Ok(())
}