use super::engine_ctx::EngineCtx;
use super::engine_queue::EngineQueue;
use super::engine_task::{EngineTask, EngineTaskHandle};
use super::entity_database::EntityDatabase;
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

pub struct EngineInternalState {
    current_frame: usize,
}

impl EngineInternalState {
    pub fn new() -> Self {
        Self { current_frame: 0 }
    }
}

pub struct Engine {
    pub title: String,
    pub development_mode: bool,
    pub local_storage: core::LocalStorage,

    pub internal_state: Mutex<EngineInternalState>,
    pub tasks: Mutex<Vec<EngineTaskHandle>>,
    pub database: Mutex<EntityDatabase>,
}

impl Engine {
    pub fn new(title: String, development_mode: bool) -> Arc<Self> {
        Arc::new(Self {
            title,
            development_mode,
            local_storage: core::LocalStorage::new(),
            internal_state: Mutex::new(EngineInternalState::new()),
            tasks: Mutex::new(Vec::new()),
            database: Mutex::new(EntityDatabase::new()),
        })
    }

    pub fn init(self: &Arc<Engine>, f: impl FnOnce(&mut EngineQueue)) {
        let mut queue = EngineQueue::new();
        f(&mut queue);

        let tasks = &mut self.tasks.lock().unwrap();
        for imp in queue.tasks.drain(..) {
            tasks.push(EngineTaskHandle::new_boxed(imp));
        }
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
                database: EntityDatabase::new(),
            }
        };
        ctx
    }

    pub fn run_frame(&self, window: EngineWindow) -> Result<(), EngineError> {
        let mut ctx = self.make_context(window);

        let mut tasks = Vec::new();
        self.swap_tasks(&mut tasks);

        // Swap the contents of the engine database and context database
        {
            let mut db = self.database.lock().unwrap();
            ctx.database.swap_entities(&mut db);
        }

        let mut next_tasks = Vec::with_capacity(tasks.len());
        for mut task_handle in tasks {
            if task_handle.run_frame(&mut ctx) {
                next_tasks.push(task_handle);
            }
            if ctx.queue.entities.len() > 0 {
                ctx.database.append(&mut ctx.queue.entities);
            }
        }
        {
            let mut db = self.database.lock().unwrap();
            ctx.database.swap_entities(&mut db);
        }
        for imp in ctx.queue.tasks.drain(..) {
            next_tasks.push(EngineTaskHandle::new_boxed(imp));
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
