// While we're in early prototyping, allow unused code to reduce noise...
#![allow(unused)]

mod core;
mod engine;
mod geometry;

use engine::prelude::{
    CameraPerspective, Engine, EngineCtx, EngineTask, EngineWindow, Renderer3D, Scene3D,
};
use engine::renderer_3d::utils;
use glam::Vec3;

fn build_scene(ctx: &mut EngineCtx) {
    let camera = CameraPerspective::new();

    let mut mesh = geometry::make_debug_cube_mesh();
    mesh.vertex_selection()
        .all()
        .scale(Vec3::splat(0.5))
        .translate(Vec3::splat(0.5));

    let scene = Scene3D {
        camera,
        triangle_buffers: vec![mesh.to_triangle_buffer()],
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
    let bbox = scene.bounding_box();
    let radius = bbox.size().length() * 1.5;
    let offset = glam::Vec3::new(
        radius * (ctx.frame as f32 * 0.002).cos(),
        radius * (ctx.frame as f32 * 0.002).sin(),
        radius * 0.5,
    );

    let mut camera = &mut scene.camera;
    camera.position = bbox.center() + offset;
    camera.look_at = bbox.center();
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
