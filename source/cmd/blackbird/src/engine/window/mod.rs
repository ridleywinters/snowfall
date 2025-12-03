mod application;
mod run_event_loop;
mod soft_panic_hook;
mod window_state;

pub mod prelude {
    pub use super::run_event_loop::init;
    pub use super::run_event_loop::run_event_loop;
}

pub mod internal {
    pub use super::application::*;
    pub use super::prelude::*;
    pub use super::window_state::*;

    pub use crate::core::*;
    pub use crate::engine::prelude::Engine;

    pub use std::sync::Arc;
    pub use winit::application::ApplicationHandler;
    pub use winit::dpi::{PhysicalPosition, PhysicalSize, Position};
    pub use winit::event::WindowEvent;
    pub use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    pub use winit::window::{Window, WindowId};
}
