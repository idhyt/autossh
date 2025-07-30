use std::io::Error;
use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{
        self,
        time::{LocalTime, UtcTime},
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn setup_logging(work_dir: &Path) -> Result<(), Error> {
    let log_dir = work_dir.join("logs");
    if !log_dir.is_dir() {
        std::fs::create_dir_all(&log_dir)?;
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let debug = filter
        .max_level_hint()
        .map(|level| level >= LevelFilter::DEBUG)
        .unwrap_or(false);

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stderr)
                .with_target(debug)
                .with_line_number(debug)
                .with_timer(LocalTime::rfc_3339()),
        )
        .with(
            fmt::Layer::new()
                .json()
                .with_writer(
                    RollingFileAppender::builder()
                        .rotation(Rotation::DAILY)
                        // .filename_prefix("atsh.log")
                        .filename_suffix("json")
                        .build(log_dir)
                        .expect("Failed to create log file"),
                )
                .with_target(debug)
                .with_line_number(debug)
                .with_timer(UtcTime::rfc_3339()),
        );

    subscriber.init();
    tracing::debug!(
        "Logging initialized (RUST_LOG={})",
        std::env::var("RUST_LOG").unwrap_or_default()
    );
    Ok(())
}
