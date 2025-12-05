// While we're in early prototyping, allow unused code to reduce noise...
#![allow(unused)]

mod core;
mod engine;

use engine::prelude::{
    CameraPerspective, Engine, EngineCtx, EngineTask, EngineWindow, Renderer3D, Scene3D,
};
use engine::renderer_3d::utils;

fn build_scene(ctx: &mut EngineCtx) {
    let camera = CameraPerspective::new();
    let mesh = utils::make_debug_cube();
    let scene = Scene3D {
        camera,
        triangle_buffers: vec![mesh],
    };
    ctx.queue.entities.push(Box::new(scene));
}

fn setup_renderer(ctx: &mut EngineCtx) {
    let mut renderer = Renderer3D::new(ctx.window.clone());
    let closure = move |ctx: &mut engine::prelude::EngineCtx| {
        let scene = ctx
            .database
            .select_mut::<Scene3D>()
            .expect("No Scene3D found in database");

        renderer.render_scene(scene);
        true
    };
    ctx.queue.task_once(|ctx| {
        println!("Renderer3D initialized.");
    });
    ctx.queue.task_frame(closure);
}

fn rotate_camera(ctx: &mut EngineCtx) -> bool {
    let scene = ctx.database.must_select_mut::<Scene3D>();

    let mut camera = &mut scene.camera;
    let radius = 4.0;
    camera.position = glam::Vec3::new(
        radius * (ctx.frame as f32 * 0.002).cos(),
        radius * (ctx.frame as f32 * 0.002).sin(),
        2.0,
    );
    camera.look_at = glam::Vec3::ZERO;
    camera.world_up = glam::Vec3::Z;
    camera.aspect_ratio = ctx.surface_width as f32 / ctx.surface_height as f32;
    true
}

fn main() {
    let engine = Engine::new("Snowfall (blackbird)".into(), true);
    println!("{}", engine.title);
    engine.init(|mut q| {
        q.task_once(build_scene);
        q.task_frame(rotate_camera);
        q.task_once(setup_renderer);
    });
    engine.run();
}
