use eyre::*;
use serde::*;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Detail,
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
            LogLevel::Detail => LevelFilter::TRACE,
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
            "detail" => Ok(LogLevel::Detail),
            "off" => Ok(LogLevel::Off),
            _ => Err(eyre!("Invalid log level: {}", s)),
        }
    }
}

fn build_env_filter(log_level: LogLevel) -> Result<EnvFilter> {
    let mut filter =
        EnvFilter::from_default_env().add_directive(log_level.as_level_filter().into());
    if log_level != LogLevel::Detail {
        filter = filter
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
    }
    Ok(filter)
}

macro_rules! build_fmt_layer {
    () => {{
        let thread_names = false;
        let target = true;
        let file_name = false;
        let line_number = true;

        let format = fmt::format::format()
            .with_thread_names(thread_names)
            .with_target(target)
            .with_file(file_name)
            .with_line_number(line_number);

        fmt::layer().event_format(format)
    }};
}

pub fn setup_logs(log_level: LogLevel) -> Result<()> {
    color_eyre::install()?;

    let filter_layer = build_env_filter(log_level)?;

    let fmt_layer = build_fmt_layer!();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
    log_panics::init();
    Ok(())
}

pub fn setup_logs_with_console_subscriber(log_level: LogLevel) -> Result<()> {
    color_eyre::install()?;

    let filter_layer = build_env_filter(log_level)?;

    let fmt_layer = build_fmt_layer!();
    let console_layer = console_subscriber::ConsoleLayer::builder().spawn();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(console_layer)
        .init();
    log_panics::init();
    Ok(())
}
