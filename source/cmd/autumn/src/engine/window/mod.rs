mod init_event_loop;

pub mod prelude {
    pub use super::init_event_loop::init_event_loop;
}

pub mod internal {
    pub use super::prelude::*;

    pub use crate::core::*;

    pub use std::sync::Arc;
    pub use winit::application::ApplicationHandler;
    pub use winit::dpi::{PhysicalPosition, PhysicalSize, Position};
    pub use winit::event::WindowEvent;
    pub use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    pub use winit::window::{Window, WindowId};
}
