use crate::load_env_recursively;
use clap::Parser;
use eyre::*;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::env::current_dir;
use std::fmt::Debug;
use std::path::PathBuf;

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
    /// The path to config file
    #[clap(long)]
    config_entry: Option<String>,
}

pub fn load_config<Config: DeserializeOwned + Debug>(
    service_name: impl AsRef<str>,
) -> Result<Config> {
    load_config_with_default_path(service_name, "etc/config.json")
}

pub fn load_config_with_default_path<Config: DeserializeOwned + Debug>(
    service_name: impl AsRef<str>,
    path: impl AsRef<str>,
) -> Result<Config> {
    println!("Loading environment");
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
    let config: Value = serde_json::from_str(&config)?;
    if let Some(entry) = args.config_entry {
        parse_config(config, &entry)
    } else {
        parse_config(config, service_name.as_ref())
    }
}

pub fn parse_config<Config: DeserializeOwned + Debug>(
    mut config: Value,
    service_name: &str,
) -> Result<Config> {
    let service_config = config
        .get_mut(&service_name)
        .ok_or_else(|| eyre!("Service {} not found in config", service_name))?
        .clone();
    let root = config.as_object_mut().unwrap();
    for (k, v) in service_config.as_object().unwrap() {
        root.insert(k.clone(), v.clone());
    }
    if service_config.get(service_name).is_none() {
        root.remove(service_name);
    }
    root.insert("name".to_string(), Value::String(service_name.to_string()));
    let config: Config = serde_json::from_value(config)?;
    println!("App config {:#?}", config);
    Ok(config)
}
