use super::engine_ctx::EngineCtx;
use std::cell::RefCell;

pub trait EngineTask {
    fn run_frame(&mut self, _ctx: &mut EngineCtx) -> bool {
        false
    }
}

impl<T: EngineTask + 'static> From<T> for Box<dyn EngineTask> {
    fn from(task: T) -> Self {
        Box::new(task)
    }
}

pub struct EngineTaskHandle {
    imp: RefCell<Box<dyn EngineTask>>,
}

impl EngineTaskHandle {
    pub fn new<T: EngineTask + 'static>(imp: T) -> Self {
        Self {
            imp: RefCell::new(Box::new(imp)),
        }
    }

    pub fn new_boxed(imp: Box<dyn EngineTask>) -> Self {
        Self {
            imp: RefCell::new(imp),
        }
    }

    pub fn run_frame(&self, ctx: &mut EngineCtx) -> bool {
        self.imp.borrow_mut().run_frame(ctx)
    }
}
