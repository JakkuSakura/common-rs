use std::env::current_dir;
use std::fmt::Debug;
use std::path::PathBuf;

use clap::Parser;
use eyre::{bail, Result};
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
    env = "CONFIG",
    value_hint = clap::ValueHint::FilePath
    )]
    config: Option<PathBuf>,
}

pub fn load_config<Config: DeserializeOwned + Debug>(path: impl AsRef<str>) -> Result<Config> {
    if let Err(err) = load_env_recursively() {
        println!("Failed to load environment: {}", err);
    }
    // print all environment variables
    // for (key, value) in std::env::vars() {
    //     println!("{}: {}", key, value);
    // }
    let args: CliArgument = CliArgument::parse();

    println!("Working directory {}", current_dir()?.display());
    let config = args.config.unwrap_or_else(|| PathBuf::from(path.as_ref()));
    println!("Loading config from {}", config.display());
    let config = std::fs::read_to_string(&config)?;
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
