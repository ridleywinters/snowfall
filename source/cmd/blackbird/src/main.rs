mod core;
mod engine;

use engine::prelude::Engine;

fn main() {
    let engine = Engine::new("Snowfall (blackbird)".into(), true);
    println!("{}", engine.title);
    engine.task_once(|_ctx| {
        println!("Engine started!");
    });
    engine.task_frame(|ctx| {
        println!("Frame {}", ctx.frame);
        true
    });
    engine.run();
}
