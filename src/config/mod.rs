pub mod prompt;
pub use prompt::*;

pub mod cli;
pub use cli::*;

use bsv::PrivateKey;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use toml::de::Error as TomlError;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MinerIDConfig {
    pub enabled: bool,
    pub priv_key: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
    pub miner_id: MinerIDConfig,
    pub pay_to: String,
    pub autopublish: bool,
}

#[derive(Deserialize)]
pub struct PolynymResponse {
    address: String,
}

impl Default for MinerConfig {
    fn default() -> Self {
        MinerConfig {
            pay_to: String::from(""),
            autopublish: true,
            miner_id: {
                MinerIDConfig {
                    enabled: false,
                    priv_key: PrivateKey::from_random().to_wif().unwrap(),
                    message: String::from(""),
                }
            },
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
impl MinerConfig {
    pub fn new(
        pay_to: String,
        autopublish: bool,
        enabled: bool,
        priv_key: String,
        message: String,
    ) -> Self {
        MinerConfig {
            pay_to,
            autopublish,
            miner_id: {
                MinerIDConfig {
                    enabled,
                    priv_key,
                    message,
                }
            },
        }
    }

    fn to_formatted_string(
        pay_to: &str,
        autopublish: &str,
        enabled: &str,
        priv_key: &str,
        message: &str,
    ) -> String {
        format!(
            concat!(
                "# Pay solved puzzles out to P2PKH address\n",
                "pay_to = \"{}\"\n",
                "# Automatically publish solved puzzles\n",
                "autopublish = {}\n",
                "\n[miner_id]\n",
                "# Enable Miner API\n",
                "enabled = {}\n",
                "# Private key in WIF format\n",
                "priv_key = \"{}\"\n",
                "# Select a message for Miner API\n",
                "message = \"{}\""
            ),
            pay_to, autopublish, enabled, priv_key, message
        )
    }

    fn to_toml_string(&self) -> String {
        MinerConfig::to_formatted_string(
            &self.pay_to,
            &self.autopublish.to_string(),
            &self.miner_id.enabled.to_string(),
            &self.miner_id.priv_key,
            &self.miner_id.message,
        )
    }

    pub fn from_toml_str(s: &str) -> Result<MinerConfig, TomlError> {
        toml::from_str::<MinerConfig>(s)
    }

    pub fn to_toml_bytes(self) -> Vec<u8> {
        self.to_toml_string().into_bytes()
    }

    pub fn existing_config() -> bool {
        Path::new("Config.toml").exists()
    }

    pub fn read_from_toml() -> Result<MinerConfig, TomlError> {
        let mut file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("Config.toml")
            .expect("Could not open Config.toml");

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        MinerConfig::from_toml_str(&content)
    }

    pub fn write_to_toml(config: MinerConfig) {
        let mut file = File::create("Config.toml").unwrap();
        file.write_all(&config.to_toml_bytes()).unwrap();
    }

    pub fn get_address(input: &str) -> String {
        let url = format!("https://api.polynym.io/getAddress/{}", input);
        let p2pkh_address = reqwest::blocking::get(url)
            .unwrap()
            .json::<PolynymResponse>()
            .unwrap();
        p2pkh_address.address
    }

    fn optional_setup() -> MinerConfig {
        match Prompt::confirm_prompt("Would you like to set up miner ID?") {
            true => Prompt::run_setup(),
            false => MinerConfig::default(),
        }
    }

    pub fn start() {
        match MinerConfig::existing_config() {
            true => println!("Found existing config."),
            false => {
                MinerConfig::optional_setup();
            }
        };
    }
}
