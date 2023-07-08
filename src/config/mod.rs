use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Read;
use toml::de::Error as TomlError;

pub struct MinerConfig {}

#[derive(Debug, Serialize, Deserialize)]
struct MinerIDConfig {
    enabled: bool,
    priv_key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    miner_id: MinerIDConfig,
    pay_to: String,
    autopublish: bool,
}

#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
impl MinerConfig {
    pub fn get_config() -> Result<ConfigFile, TomlError> {
        let mut file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("config.toml")
            .unwrap();

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        toml::from_str::<ConfigFile>(&content)
    }
}
