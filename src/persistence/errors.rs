use crate::persistence::identifier::PersistenceId;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("PersistenceModule does not exist in extension in ActorSystem.")]
    Missing,
    
    #[error("")]
    Provider,
    
    #[error("{id}")]
    NotFound {
        id: PersistenceId
    },
    
    #[error("")]
    Serialization
}