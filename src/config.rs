use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{KrxCliError, Result};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub auth_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPaths {
    pub dir: PathBuf,
    pub file: PathBuf,
}

pub fn config_paths() -> Result<ConfigPaths> {
    let home = detect_home_dir()?;
    Ok(config_paths_from_home(&home))
}

pub fn load_config() -> Result<AppConfig> {
    let paths = config_paths()?;
    load_config_from_file(&paths.file)
}

pub fn save_config(config: &AppConfig) -> Result<ConfigPaths> {
    let paths = config_paths()?;
    save_config_to_file(&paths.file, config)?;
    Ok(paths)
}

pub fn set_auth_key(auth_key: &str) -> Result<ConfigPaths> {
    validate_auth_key(auth_key)?;
    let mut config = load_config()?;
    config.auth_key = Some(auth_key.to_string());
    save_config(&config)
}

pub fn clear_auth_key() -> Result<ConfigPaths> {
    let mut config = load_config()?;
    config.auth_key = None;
    save_config(&config)
}

pub fn mask_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        return "*".repeat(secret.len());
    }

    let suffix = &secret[secret.len() - 4..];
    format!("{}{}", "*".repeat(secret.len() - 4), suffix)
}

fn detect_home_dir() -> Result<PathBuf> {
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }

    if let Some(user_profile) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(user_profile));
    }

    match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        (Some(drive), Some(path)) => {
            let mut home = PathBuf::from(drive);
            home.push(path);
            Ok(home)
        }
        _ => Err(KrxCliError::HomeDirNotFound),
    }
}

fn config_paths_from_home(home: &Path) -> ConfigPaths {
    let dir = home.join(".config").join("krx");
    let file = dir.join("config.json");
    ConfigPaths { dir, file }
}

fn load_config_from_file(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_config_to_file(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serde_json::to_string_pretty(config)?;
    fs::write(path, format!("{body}\n"))?;
    Ok(())
}

fn validate_auth_key(auth_key: &str) -> Result<()> {
    if auth_key.trim().is_empty() {
        return Err(KrxCliError::InvalidInput(
            "auth key must not be empty".to_string(),
        ));
    }

    if auth_key.chars().any(|char| char.is_control()) {
        return Err(KrxCliError::InvalidInput(
            "auth key contains control characters".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn config_path_uses_dot_config_krx_under_home() {
        let paths = config_paths_from_home(Path::new("/tmp/example-home"));
        assert_eq!(paths.dir, PathBuf::from("/tmp/example-home/.config/krx"));
        assert_eq!(
            paths.file,
            PathBuf::from("/tmp/example-home/.config/krx/config.json")
        );
    }

    #[test]
    fn save_and_load_config_round_trip() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("krx-cli-config-test-{unique}"));
        let file = root.join("config.json");

        let config = AppConfig {
            auth_key: Some("secret-key".to_string()),
        };
        save_config_to_file(&file, &config).unwrap();
        let loaded = load_config_from_file(&file).unwrap();

        assert_eq!(loaded.auth_key.as_deref(), Some("secret-key"));

        fs::remove_dir_all(&root).ok();
    }
}
