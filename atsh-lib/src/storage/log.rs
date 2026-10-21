use std::io::Error;
use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn setup_logging(work_dir: &Path) -> Result<(), Error> {
    let log_dir = work_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("russh=info".parse().expect("static directive"));

    // Show source location only when the effective level is debug or lower.
    let show_location = filter
        .max_level_hint()
        .is_none_or(|level| level <= tracing_subscriber::filter::LevelFilter::DEBUG);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stderr)
                .with_target(show_location)
                .with_line_number(show_location)
                .with_timer(LocalTime::rfc_3339()),
        )
        .with(
            fmt::Layer::new()
                .json()
                .with_writer(
                    RollingFileAppender::builder()
                        .rotation(Rotation::DAILY)
                        .filename_suffix("json")
                        .build(log_dir)
                        .expect("Failed to create log file"),
                )
                .with_target(show_location)
                .with_line_number(show_location)
                .with_timer(LocalTime::rfc_3339()),
        );

    subscriber.init();
    tracing::debug!(
        "logging initialized (RUST_LOG={})",
        std::env::var("RUST_LOG").unwrap_or_default()
    );
    Ok(())
}