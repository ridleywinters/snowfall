use super::internal::*;

pub fn init_event_loop() {
    let event_loop = {
        let mut builder = EventLoop::builder();

        // TODO: this needs to be conditional behind a --development flag What it does it
        // ensures when the new window is created, it will **not** steal focus, which is
        // very useful for auto-recompiling so the focus is not stolen from the code
        // editor.
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            builder.with_activate_ignoring_other_apps(false);
        }

        builder.build().unwrap()
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AutumnApp { window: None };
    event_loop.run_app(&mut app).unwrap();

    println!("Event loop initialized");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct WindowState {
    position: (i32, i32),
    size: (u32, u32),
}

struct AutumnApp {
    // The winit window
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for AutumnApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_state = match local_storage_get::<WindowState>("window_state") {
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
            .with_title("Autumn")
            .with_active(false)
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
        // This is somewhat redundant since RedrawRequested also calls
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
                local_storage_set(
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
                };

                let size = window.inner_size();
                handler
                    .run_frame(size.width as usize, size.height as usize)
                    .expect("Failed to run frame");*/
                let Some(window) = self.window.as_ref() else {
                    return;
                };
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {}
            WindowEvent::CursorMoved { position, .. } => {}
            WindowEvent::MouseInput { state, button, .. } => {}
            WindowEvent::MouseWheel { delta, .. } => {}
            _ => (),
        }
    }
}
