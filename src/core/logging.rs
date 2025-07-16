use std::fs::OpenOptions;
use backtrace::Backtrace;
use tracing_subscriber::layer::SubscriberExt;

pub fn initialize_logging() {
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("mc_session_changer.log")
        .expect("Failed to create log file");

    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_writer(log_file);

    let subscriber = tracing_subscriber::Registry::default().with(stdout_log);
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        tracing::error!("PANIC: {}", panic_info);
        tracing::error!("Backtrace: {:?}", backtrace);
    }));
}