use std::fs::OpenOptions;
use std::io::Write;
use backtrace::Backtrace;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use tracing_subscriber::fmt::time::ChronoUtc;

pub fn initialize_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("mc_session_changer.log")?;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_timer(ChronoUtc::rfc_3339())
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_file(true);

    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();

        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown".to_string());

        tracing::error!(
            "PANIC occurred at {}: {}",
            location,
            panic_info.payload()
                .downcast_ref::<&str>()
                .unwrap_or(&"unknown panic")
        );

        tracing::error!("Backtrace:\n{:?}", backtrace);

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("panic.log") {
            let _ = writeln!(file, "[{}] PANIC at {}: {}",
                             chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                             location,
                             panic_info.payload()
                                 .downcast_ref::<&str>()
                                 .unwrap_or(&"unknown panic")
            );
        }
    }));

    tracing::info!("Logging initialized successfully");
    Ok(())
}