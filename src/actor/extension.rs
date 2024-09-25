use std::any::type_name;
use crate::actor::{Context, FromContext};
use crate::errors::ActorError;
use crate::system::ExtensionMissingError;

pub struct Extension<T>(pub T);

#[async_trait::async_trait]
impl<T> FromContext for Extension<T> 
    where T: Clone + Sync + Send + 'static
{
    type Rejection = ActorError;
    
    async fn from_context(ctx: &mut Context) -> Result<Self, Self::Rejection> {
        let ext = ctx
            .system()
            .extension()
            .get::<T>()
            .ok_or_else(|| ActorError::MissingExtension(ExtensionMissingError {
                module: type_name::<T>()
            }))
            .cloned()?;
        Ok(Extension(ext))
    }
}