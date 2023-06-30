use eyre::*;
use serde::*;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
impl LogLevel {
    pub fn as_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}
impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            "off" => Ok(LogLevel::Off),
            _ => Err(eyre!("Invalid log level: {}", s)),
        }
    }
}
fn build_env_filter(log_level: LogLevel) -> Result<EnvFilter> {
    let filter = EnvFilter::from_default_env()
        .add_directive(log_level.as_level_filter().into())
        .add_directive("tungstenite::protocol=debug".parse()?)
        .add_directive("tokio_postgres::connection=debug".parse()?)
        .add_directive("tokio_util::codec::framed_impl=debug".parse()?)
        .add_directive("tokio_tungstenite=debug".parse()?)
        .add_directive("h2=info".parse()?)
        .add_directive("rustls::client::hs=info".parse()?)
        .add_directive("rustls::client::tls13=info".parse()?)
        .add_directive("hyper::client=info".parse()?)
        .add_directive("hyper::proto=info".parse()?)
        .add_directive("mio=info".parse()?)
        .add_directive("want=info".parse()?);
    Ok(filter)
}
pub fn setup_logs(log_level: LogLevel) -> Result<()> {
    LogTracer::init().context("Cannot setup_logs")?;
    color_eyre::config::HookBuilder::new()
        .panic_section("ERROR|FATAL program panicked")
        .install()?;
    let filter = build_env_filter(log_level)?;

    let subscriber = fmt()
        .with_thread_names(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).context("Cannot setup_logs")?;
    log_panics::init();
    Ok(())
}
