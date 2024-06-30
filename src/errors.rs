use crate::identifier::ActorId;

#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("Actor with this identifier: {id} has already been activated.")]
    AlreadySpawned {
        id: ActorId
    },
    
    #[error("The target actor could not be found. actor: `{id}` It may have already been shut down or may not have started.")]
    NotFoundActor {
        id: ActorId
    },

    #[error("")]
    CallBackSend,

    #[error("May have passed different type information than what was expected when downcasting from `Any` to type.")]
    DownCastFromAny,
    
    #[error("")]
    MissingExtension {
        ext: &'static str
    },
}
