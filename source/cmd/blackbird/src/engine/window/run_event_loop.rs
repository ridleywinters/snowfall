use super::internal::*;

pub fn init() {}

pub fn run_event_loop(engine: Arc<Engine>) {
    let event_loop = {
        let mut builder = EventLoop::builder();

        if engine.development_mode {
            // This ensures when the new window is created, it will **not** steal focus,
            // which is very useful for auto-recompiling so the focus is not stolen from
            // the code editor.
            #[cfg(target_os = "macos")]
            {
                use winit::platform::macos::EventLoopBuilderExtMacOS;
                builder.with_activate_ignoring_other_apps(false);
            }

            // A custom panic handler is made available (though opt-in) to provide a
            // work-around for the MacOS dialog that pops up when a regular panic. This
            // can be quite intrusive during iterative local development.
            std::panic::set_hook(Box::new(soft_panic_hook));
        }

        builder.build().unwrap()
    };
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(engine.clone());
    event_loop.run_app(&mut app).unwrap();
}

/// Custom panic implementation that does not trigger the MacOS native dialog.
///
fn soft_panic_hook(info: &std::panic::PanicHookInfo) {
    println!();
    println!("❌ panic occurred");

    match info.location() {
        Some(location) => {
            let file = location.file();
            let line = location.line();
            println!("{}:{}", file, line);
        }
        None => println!("panic at: <unknown location>"),
    }

    println!();

    let payload = info.payload();
    if let Some(string) = payload.downcast_ref::<String>() {
        eprintln!("{string}");
    } else if let Some(str) = payload.downcast_ref::<&'static str>() {
        eprintln!("{str}")
    } else {
        eprintln!("{payload:?}")
    }

    let backtrace = std::backtrace::Backtrace::force_capture();
    let display = format!("{:?}", backtrace);
    let display = match display.find('[') {
        Some(idx) => &display[idx + 1..],
        None => &display,
    }
    .to_string();
    let display = match display.rfind(']') {
        Some(idx) => &display[..idx],
        None => &display,
    }
    .to_string();

    let lines: Vec<&str> = display.split(',').map(|line| line.trim()).collect();
    let lines: Vec<&str> = lines
        .iter()
        .filter_map(|line| {
            let start = line.find("{ fn: \"")?;
            let rest = &line[start + 7..];
            let end = rest.find('"')?;
            Some(&rest[..end])
        })
        .collect();
    let last_panicking_idx = lines
        .iter()
        .rposition(|line| line.contains("core::panicking"));
    let lines = match last_panicking_idx {
        Some(idx) => lines.into_iter().skip(idx + 1).collect(),
        None => lines,
    };

    for line in lines.iter() {
        println!("{}", line);
    }
    println!();

    std::process::exit(1);
}
