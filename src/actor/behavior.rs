use crate::actor::IntoActor;
use crate::actor::lifecycle::Runnable;


pub struct Behavior<T: IntoActor>(T);

impl<T: IntoActor> Runnable for Behavior<T> {}

impl<T: IntoActor> Behavior<T> {
    pub fn from_adaptor(adaptor: T) -> Self {
        Self(adaptor)
    }
}

impl<T: IntoActor> IntoActor for Behavior<T> {
    type Actor = T::Actor;
    fn into_actor(self) -> Self::Actor {
        self.0.into_actor()
    }
}


pub trait IntoBehavior: 'static + Sync + Send + Sized 
    where Self: IntoActor
{
    fn into_behavior(self) -> Behavior<Self>;
}

impl<T: IntoActor> IntoBehavior for T {
    fn into_behavior(self) -> Behavior<Self> {
        Behavior::from_adaptor(self)
    }
}

pub trait Adaptor: 'static + Sync + Send {
    type Adaptor;
    fn adaptor(self) -> Self::Adaptor;
}
