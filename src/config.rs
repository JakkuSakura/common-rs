use std::env::current_dir;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use clap::Parser;
use eyre::{bail, Context, Result};
use serde::de::DeserializeOwned;

use crate::load_env_recursively;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgument {
    /// The path to config file
    #[clap(
    short,
    long,
    value_parser,
    value_name = "FILE",
    value_hint = clap::ValueHint::FilePath
    )]
    config: Option<PathBuf>,
}

pub fn load_config_with_cli<Config: DeserializeOwned + Debug>(
    path: impl AsRef<Path>,
) -> Result<Config> {
    load_env_recursively()?;
    let args: CliArgument = CliArgument::parse();
    let config = args.config.unwrap_or_else(|| PathBuf::from(path.as_ref()));
    load_config(config)
}
pub fn load_config<Config: DeserializeOwned + Debug>(path: impl AsRef<Path>) -> Result<Config> {
    println!("Working directory {}", current_dir()?.display());
    let path = path.as_ref();
    println!("Loading config from {}", path.display());
    let config =
        std::fs::read_to_string(&path).with_context(|| format!("path: {}", path.display()))?;
    parse_config(&config)
}

pub fn parse_config<Config: DeserializeOwned + Debug>(content: &str) -> Result<Config> {
    let jd = &mut serde_json::Deserializer::from_str(content);

    let result: Result<Config, _> = serde_path_to_error::deserialize(jd);
    let config = match result {
        Ok(config) => config,
        Err(err) => {
            let path = err.path().to_string();
            bail!("Failed to parse config at {}: {}", path, err.inner())
        }
    };
    println!("App config {:#?}", config);
    Ok(config)
}
