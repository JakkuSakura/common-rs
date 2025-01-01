use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use eyre::{eyre, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogRotationInterval {
    #[default]
    Never,
    Daily,
    Hourly,
}
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogConsoleTarget {
    Null,
    #[default]
    Stdout,
    Stderr,
}
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
    type Err = eyre::Error;

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
            .add_directive("tungstenite::handshake=debug".parse()?)
            .add_directive("tokio_postgres::connection=debug".parse()?)
            .add_directive("tokio_util::codec::framed_impl=debug".parse()?)
            .add_directive("databend_client=warn".parse()?)
            .add_directive("databend_driver=warn".parse()?)
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

// type of format is too complex to be built with a function
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

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: LogLevel,
    #[serde(default)]
    pub console: LogConsoleTarget,
    pub file: Option<PathBuf>,
    pub rotation: LogRotationInterval,
    pub console_subscriber: bool,
}
impl From<LogLevel> for LogConfig {
    fn from(level: LogLevel) -> Self {
        Self {
            level,
            console: LogConsoleTarget::Stdout,
            file: None,
            rotation: LogRotationInterval::Never,
            console_subscriber: false,
        }
    }
}
impl LogConfig {
    fn console_layer<S>(&self) -> Option<Box<dyn Layer<S> + Send + Sync>>
    where
        S: Subscriber,
        for<'a> S: LookupSpan<'a>,
    {
        match self.console {
            LogConsoleTarget::Null => None,
            LogConsoleTarget::Stdout => {
                Some(build_fmt_layer!().with_writer(std::io::stdout).boxed())
            }
            LogConsoleTarget::Stderr => {
                Some(build_fmt_layer!().with_writer(std::io::stderr).boxed())
            }
        }
    }
    fn file_layer<S>(&self) -> Result<Option<Box<dyn Layer<S> + Send + Sync>>>
    where
        S: Subscriber,
        for<'a> S: LookupSpan<'a>,
    {
        let Some(path) = self.file.as_ref() else {
            return Ok(None);
        };

        let dir = path
            .parent()
            .ok_or_else(|| eyre!("Invalid log file path"))?;
        std::fs::create_dir_all(dir).with_context(|| format!("Failed to create dir: {:?}", dir))?;
        let prefix = path
            .file_stem()
            .ok_or_else(|| eyre!("Invalid log file path"))?;
        let rotation = match self.rotation {
            LogRotationInterval::Daily => tracing_appender::rolling::daily(path, prefix),
            LogRotationInterval::Hourly => tracing_appender::rolling::hourly(path, prefix),
            LogRotationInterval::Never => tracing_appender::rolling::never(path, prefix),
        };
        Ok(Some(
            build_fmt_layer!()
                .with_ansi(false)
                .with_writer(rotation)
                .boxed(),
        ))
    }
    fn console_subscriber_layer<S>(&self) -> Option<impl Layer<S> + Send + Sync>
    where
        S: Subscriber,
        for<'a> S: LookupSpan<'a>,
    {
        if self.console_subscriber {
            Some(console_subscriber::ConsoleLayer::builder().spawn())
        } else {
            None
        }
    }
    pub fn install(&self) -> Result<()> {
        // avoid output capturing in tests
        writeln!(std::io::stdout(), "Setting up logs: {:?}", self)?;
        color_eyre::install()?;

        let filter_layer = build_env_filter(self.level)?;

        let file_layer = self.file_layer()?;

        let console_layer = self.console_layer();

        let console_subscriber_layer = self.console_subscriber_layer();

        // special handling to make it fast
        if console_layer.is_some() && file_layer.is_none() && console_subscriber_layer.is_none() {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(console_layer.unwrap())
                .init();
        } else {
            tracing_subscriber::registry()
                .with(filter_layer)
                .with(console_layer)
                .with(file_layer)
                .with(console_subscriber_layer)
                .init();
        }

        log_panics::init();

        Ok(())
    }
}
pub fn setup_logs(log: impl Into<LogConfig>) -> Result<()> {
    log.into().install()
}
pub fn get_log_level_from_verbosity(verbosity: u8, levels: &[LogLevel]) -> LogLevel {
    if verbosity >= levels.len() as u8 {
        levels.last().copied().unwrap()
    } else {
        levels[verbosity as usize]
    }
}
