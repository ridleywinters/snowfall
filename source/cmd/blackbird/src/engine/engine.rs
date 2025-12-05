use crate::core;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use wgpu::hal::auxil::db::qualcomm;
use wgpu::wgc::device::queue;
use winit::window::Window;

#[derive(Debug, Clone)]
pub enum EngineError {
    Generic(String),
}

pub type EngineWindow = Arc<Window>;

pub struct Engine {
    pub title: String,
    pub development_mode: bool,
    pub local_storage: core::LocalStorage,

    pub internal_state: Mutex<EngineInternalState>,
    pub tasks: Mutex<Vec<EngineTaskHandle>>,
}

impl Engine {
    pub fn new(title: String, development_mode: bool) -> Arc<Self> {
        Arc::new(Self {
            title,
            development_mode,
            local_storage: core::LocalStorage::new(),
            internal_state: Mutex::new(EngineInternalState::new()),
            tasks: Mutex::new(Vec::new()),
        })
    }

    pub fn run(self: Arc<Engine>) {
        super::prelude::init();

        super::prelude::run_event_loop(self.clone());
    }

    fn make_context(&self, window: EngineWindow) -> EngineCtx {
        let (width, height) = {
            let size = window.inner_size();
            (size.width as usize, size.height as usize)
        };

        let mut ctx = {
            let mut state = self.internal_state.lock().unwrap();
            state.current_frame += 1;

            EngineCtx {
                frame: state.current_frame,
                surface_width: width,
                surface_height: height,
                window: window.clone(),
                queue: EngineQueue::new(),
            }
        };
        ctx
    }

    pub fn run_frame(&self, window: EngineWindow) -> Result<(), EngineError> {
        let mut ctx = self.make_context(window);

        let mut tasks = Vec::new();
        self.swap_tasks(&mut tasks);

        let mut next_tasks = Vec::with_capacity(tasks.len());
        for mut task_handle in tasks {
            if task_handle.run_frame(&mut ctx) {
                next_tasks.push(task_handle);
            }
        }
        for imp in ctx.queue.tasks.drain(..) {
            next_tasks.push(EngineTaskHandle {
                imp: RefCell::new(imp),
            });
        }
        self.swap_tasks(&mut next_tasks);

        Ok(())
    }

    fn swap_tasks(&self, other: &mut Vec<EngineTaskHandle>) {
        let mut tasks = self.tasks.lock().unwrap();
        std::mem::swap(&mut *tasks, other);
    }

    pub fn task(&self, imp: impl EngineTask + 'static) {
        let handle = EngineTaskHandle::new(imp);
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(handle);
    }

    pub fn task_once(&self, f: impl FnMut(&mut EngineCtx) + 'static) {
        struct TaskOnce<F: FnMut(&mut EngineCtx)> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&mut EngineCtx)> EngineTask for TaskOnce<F> {
            fn run_frame(&mut self, ctx: &mut EngineCtx) -> bool {
                (self.f.borrow_mut())(ctx);
                false
            }
        }
        let handle = EngineTaskHandle::new(TaskOnce { f: RefCell::new(f) });
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

pub struct EngineQueue {
    pub tasks: Vec<Box<dyn EngineTask>>,
}

impl EngineQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn task(&mut self, imp: Box<dyn EngineTask>) {
        self.tasks.push(imp);
    }

    pub fn task_once(&mut self, f: impl FnMut(&EngineCtx) + 'static) {
        struct TaskOnce<F: FnMut(&EngineCtx)> {
            f: RefCell<F>,
        }
        impl<F: FnMut(&EngineCtx)> EngineTask for TaskOnce<F> {
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

pub struct EngineCtx {
    pub frame: usize,
    pub surface_width: usize,
    pub surface_height: usize,

    pub window: EngineWindow,
    pub queue: EngineQueue,
}

pub trait EngineTask {
    fn run_frame(&mut self, _ctx: &mut EngineCtx) -> bool {
        false
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

    pub fn run_frame(&self, ctx: &mut EngineCtx) -> bool {
        self.imp.borrow_mut().run_frame(ctx)
    }
}
