/// Application is the concrete implementation that implements the
/// `ApplicationHandler` trait to handle events from the event loop.
///
/// It is intended to as fully as possible encapsulate the winit implementation
/// details so the user of the engine does not need to directly interact with
/// winit.
///
pub use super::internal::*;

pub struct Application {
    // The winit window
    window: Option<Arc<Window>>,
    engine: Arc<Engine>,
}

impl Application {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self {
            window: None,
            engine,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_state = match self.engine.local_storage.get::<WindowState>("window_state") {
            Some(state) => state,
            None => WindowState {
                position: (0, 0),
                size: (800, 600),
            },
        };
        let position = Position::Physical(PhysicalPosition::new(
            window_state.position.0,
            window_state.position.1,
        ));
        let size = PhysicalSize::new(window_state.size.0, window_state.size.1);

        let window_attributes = Window::default_attributes()
            .with_title(self.engine.title.clone())
            .with_active(!self.engine.development_mode)
            .with_position(position)
            .with_inner_size(size);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // ✏️ Not sure is this is a bug in winit or a error in this code, but the position
        // given in the original window attributes is not always respected.  So we set it
        // again here.
        window.set_outer_position(position);

        self.window = Some(window.clone());
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // This is possibly redundant since RedrawRequested also calls
        // request_redraw.
        let Some(window) = self.window.as_ref() else {
            return;
        };
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Moved(position) => {
                self.engine.local_storage.set(
                    "window_state",
                    &WindowState {
                        position: (position.x, position.y),
                        size: (800, 600),
                    },
                );
            }
            WindowEvent::RedrawRequested => {
                //
                // Check we have valid objects for the redraw or early exit.
                //
                /*let (Some(window), Some(handler)) = (self.window.as_ref(), self.handler.as_mut())
                else {
                    return;
                };*/

                let Some(window) = self.window.as_ref() else {
                    return;
                };
                if let Err(e) = self.engine.run_frame(window.clone()) {
                    eprintln!("Error during frame render: {:?}", e);
                    event_loop.exit();
                    return;
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput { .. } => {}
            WindowEvent::CursorMoved { .. } => {}
            WindowEvent::MouseInput { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            _ => (),
        }
    }
}
