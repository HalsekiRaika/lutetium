use lutetium::system::ActorSystem;

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let test = "extension".to_string();
    
    let mut system = ActorSystem::builder();
    
    system.extension(move |ext| {
        ext.install(test);
    });
    
    let _system = system.build();
    
    Ok(())
}