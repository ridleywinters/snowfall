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
        let camera = CameraPerspective::new();
        let scene = Scene3D {
            camera,
            triangle_buffers: vec![mesh],
        };
        Self { renderer, scene }
    }
}

impl EngineTask for RendererTask {
    fn run_frame(&mut self, ctx: &mut engine::prelude::EngineCtx) -> bool {
        let mut camera = &mut self.scene.camera;
        let radius = 4.0;
        camera.position = glam::Vec3::new(
            radius * (ctx.frame as f32 * 0.002).cos(),
            radius * (ctx.frame as f32 * 0.002).sin(),
            2.0,
        );
        camera.look_at = glam::Vec3::ZERO;
        camera.world_up = glam::Vec3::Z;
        camera.aspect_ratio = ctx.surface_width as f32 / ctx.surface_height as f32;

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
