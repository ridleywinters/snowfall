/// Custom panic implementation that does not trigger the MacOS native dialog.
///
/// This is implemented so that during development, which uses automatic restarts
/// of the engine, the native dialog does not steal focus and require a manual click.
///
pub fn soft_panic_hook(info: &std::panic::PanicHookInfo) {
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
