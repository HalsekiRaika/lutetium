use lutetium::system::ActorSystem;

#[tokio::test]
async fn main() {
    let _system = ActorSystem::builder().build();
}