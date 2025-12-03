mod core;
mod engine;
pub use std::sync::Arc;

pub struct Engine {
    title: String,
    development_mode: bool,
    local_storage: core::LocalStorage,
}

impl Engine {
    fn run(self: Arc<Engine>) {
        engine::prelude::init();
        engine::prelude::run_event_loop(self.clone());
    }
}

fn main() {
    let engine = Arc::new(Engine {
        title: "Snowfall".into(),
        development_mode: true,
        local_storage: core::LocalStorage::new(),
    });
    println!("{}", engine.title);
    engine.run();
}
