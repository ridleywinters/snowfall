use super::engine_ctx::EngineCtx;
use super::engine_task::EngineTask;
use std::cell::RefCell;

pub struct EngineQueue {
    pub entities: Vec<Box<dyn std::any::Any>>,
    pub tasks: Vec<Box<dyn EngineTask>>,
}

impl EngineQueue {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn task(&mut self, imp: impl Into<Box<dyn EngineTask>>) {
        self.tasks.push(imp.into());
    }

    pub fn task_once(&mut self, f: impl FnMut(&mut EngineCtx) + 'static) {
        struct TaskOnce<F: FnMut(&mut EngineCtx)> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&mut EngineCtx)> EngineTask for TaskOnce<F> {
            fn run_frame(&mut self, _ctx: &mut EngineCtx) -> bool {
                (self.f.borrow_mut())(_ctx);
                false
            }
        }
        self.tasks.push(Box::new(TaskOnce { f: RefCell::new(f) }));
    }

    pub fn task_frame(&mut self, f: impl FnMut(&mut EngineCtx) -> bool + 'static) {
        struct TaskFrame<F: FnMut(&mut EngineCtx) -> bool> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&mut EngineCtx) -> bool> EngineTask for TaskFrame<F> {
            fn run_frame(&mut self, ctx: &mut EngineCtx) -> bool {
                (self.f.borrow_mut())(ctx)
            }
        }
        self.tasks.push(Box::new(TaskFrame { f: RefCell::new(f) }));
    }
}
