use std::{error::Error, path::PathBuf};
use tokio::fs::read_to_string;

use crate::{
    global_vars::{HOME, REPOS_BASE_PATH},
    types::config_toml::Config,
};

pub async fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_file = PathBuf::from("brag-server.toml");
    if !config_file.is_file() {
        panic!("brag-server.toml is not a file");
    }
    let config_str = read_to_string(config_file)
        .await
        .expect("brag-server.toml: can not read this file");
    toml::from_str(&config_str).map_err(|e| format!("Failed to parse config as TOML: {}", e).into())
}

pub fn repos_base_path() -> PathBuf {
    let path_str = format!("{}{}", HOME, REPOS_BASE_PATH);
    PathBuf::from(&path_str)
}
