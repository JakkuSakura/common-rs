use dotenvy::dotenv;
use eyre::{Context, Result};
use std::path::Path;

pub fn load_env() -> Result<()> {
    let path = std::env::current_dir()?;
    println!("Loading environment from path: {}", path.display());
    dotenv().with_context(|| "failed to load .env")?;
    Ok(())
}

/// recursively search for .env file in the current directory and its parents
/// return true if found and loaded, false otherwise
pub fn load_env_recursively() -> Result<bool> {
    let mut path = std::env::current_dir()?;
    println!(
        "Loading environment recursively from path: {}",
        path.display()
    );
    if let Ok(secrets_dir) = std::env::var("NOMAD_SECRETS_DIR") {
        let env_path = Path::new(&secrets_dir).join(".env");
        if env_path.exists() {
            println!("Loading .env from Nomad secrets path: {}", env_path.display());
            match std::fs::File::open(&env_path) {
                Ok(_) => match dotenvy::from_path(&env_path) {
                    Ok(_) => return Ok(true),
                    Err(err) => {
                        eprintln!(
                            "Failed to load .env from path: {}: {}",
                            env_path.display(),
                            err
                        );
                    }
                },
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    eprintln!(
                        "Skipping .env from {} (permission denied)",
                        env_path.display()
                    );
                }
                Err(err) => {
                    return Err(err)
                        .with_context(|| format!("Failed to open .env at {}", env_path.display()));
                }
            };
        }
    }
    loop {
        let env_path = path.join(".env");
        if env_path.exists() {
            println!("Loading .env from path: {}", env_path.display());
            dotenvy::from_path(&env_path).with_context(|| {
                format!("Failed to load .env from path: {}", env_path.display())
            })?;
            return Ok(true);
        }
        if !path.pop() {
            break;
        }
    }
    Ok(false)
}
