use std::fs::OpenOptions;
use std::io::Write;
use backtrace::Backtrace;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub fn initialize_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("mc_session_changer.log")?;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(false)
        .with_file(false);

    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();

        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let panic_msg = panic_info.payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"unknown panic");

        tracing::error!(
            "PANIC occurred at {}: {}",
            location,
            panic_msg
        );

        tracing::error!("Backtrace:\n{:?}", backtrace);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("panic.log") {
            let _ = writeln!(file, "[{}] PANIC at {}: {}",
                             chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                             location,
                             panic_msg
            );
            let _ = file.flush();
        }

        let _ = std::io::stderr().flush();

        tracing::error!("Panic hook completed - VEH should handle cleanup");
    }));

    tracing::info!("Logging initialized successfully");
    Ok(())
}
