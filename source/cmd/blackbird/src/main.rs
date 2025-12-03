// While we're in early prototyping, allow unused code to reduce noise...
#![allow(unused)]

mod core;
mod engine;

use engine::prelude::{CameraPerspective, Engine, EngineWindow, Renderer3D, Scene3D};

fn main() {
    let engine = Engine::new("Snowfall (blackbird)".into(), true);
    println!("{}", engine.title);
    engine.task_once(|ctx| {
        println!("Engine started!");
        let mut renderer = Renderer3D::new(ctx.window.clone());
        let scene = Scene3D {
            camera: CameraPerspective::new(),
        };
    });
    engine.task_frame(|_ctx| true);
    engine.run();
}
