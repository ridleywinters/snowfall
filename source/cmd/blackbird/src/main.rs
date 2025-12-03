mod core;
mod engine;

use engine::prelude::{Engine, EngineWindow, Renderer3D};

fn main() {
    let engine = Engine::new("Snowfall (blackbird)".into(), true);
    println!("{}", engine.title);
    engine.task_once(|ctx| {
        println!("Engine started!");
        let _ = Renderer3D::new(ctx.window.clone());
    });
    engine.task_frame(|_ctx| true);
    engine.run();
}
