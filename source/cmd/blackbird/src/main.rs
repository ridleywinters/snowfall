// While we're in early prototyping, allow unused code to reduce noise...
#![allow(unused)]

mod core;
mod engine;

use std::cell::RefCell;
use std::rc::Rc;

use engine::prelude::{CameraPerspective, Engine, EngineWindow, Renderer3D, Scene3D};
use engine::renderer_3d::utils;

use crate::engine::prelude::EngineTask;

struct RendererTask {
    renderer: Renderer3D,
    scene: Scene3D,
}

impl RendererTask {
    pub fn new(window: EngineWindow) -> Self {
        let renderer = Renderer3D::new(window.clone());
        let mesh = utils::make_debug_cube(&renderer.device);
        let scene = Scene3D {
            camera: CameraPerspective::new(),
            triangle_buffers: vec![mesh],
        };
        Self { renderer, scene }
    }
}

impl EngineTask for RendererTask {
    fn run_frame(&mut self, ctx: &mut engine::prelude::EngineCtx) -> bool {
        self.renderer.render_scene(&mut self.scene);
        println!("Rendered 3D frame {}", ctx.frame);
        true
    }
}

fn main() {
    let engine = Engine::new("Snowfall (blackbird)".into(), true);
    println!("{}", engine.title);

    engine.task_once(|ctx| {
        println!("Engine started!");
        let task = RendererTask::new(ctx.window.clone());
        ctx.queue.task(Box::new(task));
    });
    engine.run();
}
