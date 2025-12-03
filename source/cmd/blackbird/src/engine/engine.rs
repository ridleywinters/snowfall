use crate::core;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum EngineError {
    Generic(String),
}

pub struct Engine {
    pub title: String,
    pub development_mode: bool,
    pub local_storage: core::LocalStorage,

    pub internal_state: Mutex<EngineInternalState>,
    pub tasks: Mutex<Vec<EngineTaskHandle>>,
    pub queue: EngineQueue,
}

impl Engine {
    pub fn new(title: String, development_mode: bool) -> Arc<Self> {
        Arc::new(Self {
            title,
            development_mode,
            local_storage: core::LocalStorage::new(),
            internal_state: Mutex::new(EngineInternalState::new()),
            tasks: Mutex::new(Vec::new()),
            queue: EngineQueue {},
        })
    }

    pub fn run(self: Arc<Engine>) {
        super::prelude::init();
        super::prelude::run_event_loop(self.clone());
    }

    pub fn run_frame(&self, width: usize, height: usize) -> Result<(), EngineError> {
        let ctx = {
            let mut state = self.internal_state.lock().unwrap();
            state.current_frame += 1;

            EngineCtx {
                frame: state.current_frame,
                surface_width: width,
                surface_height: height,
                queue: &self.queue,
            }
        };

        let mut tasks = Vec::new();
        self.swap_tasks(&mut tasks);
        tasks.retain(|task_handle| task_handle.run_frame(&ctx));
        self.swap_tasks(&mut tasks);

        Ok(())
    }

    fn swap_tasks(&self, other: &mut Vec<EngineTaskHandle>) {
        let mut tasks = self.tasks.lock().unwrap();
        std::mem::swap(&mut *tasks, other);
    }

    pub fn task_once(&self, f: impl FnMut(&EngineCtx) + 'static) {
        struct TaskOnce<F: FnMut(&EngineCtx)> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&EngineCtx)> EngineTask for TaskOnce<F> {
            fn run_frame(&self, _ctx: &EngineCtx) -> bool {
                (self.f.borrow_mut())(_ctx);
                false
            }
        }
        let handle = EngineTaskHandle::new(TaskOnce { f: RefCell::new(f) });
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(handle);
    }

    pub fn task_frame(&self, f: impl FnMut(&EngineCtx) -> bool + 'static) {
        struct TaskFrame<F: FnMut(&EngineCtx) -> bool> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&EngineCtx) -> bool> EngineTask for TaskFrame<F> {
            fn run_frame(&self, ctx: &EngineCtx) -> bool {
                (self.f.borrow_mut())(ctx)
            }
        }
        let handle = EngineTaskHandle::new(TaskFrame { f: RefCell::new(f) });
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(handle);
    }
}

pub struct EngineInternalState {
    current_frame: usize,
}

impl EngineInternalState {
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }
}

pub struct EngineQueue {}

pub struct EngineCtx<'a> {
    pub frame: usize,
    pub surface_width: usize,
    pub surface_height: usize,

    pub queue: &'a EngineQueue,
}

pub trait EngineTask {
    fn run_frame(&self, _ctx: &EngineCtx) -> bool {
        false
    }
}

pub struct EngineTaskHandle {
    imp: Box<dyn EngineTask>,
}

impl EngineTaskHandle {
    pub fn new<T: EngineTask + 'static>(imp: T) -> Self {
        Self { imp: Box::new(imp) }
    }

    pub fn run_frame(&self, ctx: &EngineCtx) -> bool {
        self.imp.run_frame(ctx)
    }
}
