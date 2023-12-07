use crate::Prompt;
use crate::Res;
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
pub struct Config {
    pub miner_id: MinerIDConfig,
    pub pay_to: String,
    pub autopublish: bool,
    pub autosave: bool,
}

#[derive(Deserialize)]
pub struct PolynymResponse {
    address: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pay_to: String::from(""),
            autopublish: true,
            autosave: true,
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
impl Config {
    pub fn new(
        pay_to: String,
        autopublish: bool,
        autosave: bool,
        enabled: bool,
        priv_key: String,
        message: String,
    ) -> Self {
        Config {
            pay_to,
            autopublish,
            autosave,
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
        autosave: &str,
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
                "autosave = {}\n",
                "\n[miner_id]\n",
                "# Enable Miner API\n",
                "enabled = {}\n",
                "# Private key in WIF format\n",
                "priv_key = \"{}\"\n",
                "# Select a message for Miner API\n",
                "message = \"{}\""
            ),
            pay_to, autopublish, autosave, enabled, priv_key, message
        )
    }

    fn to_toml_string(&self) -> String {
        Config::to_formatted_string(
            &self.pay_to,
            &self.autopublish.to_string(),
            &self.autosave.to_string(),
            &self.miner_id.enabled.to_string(),
            &self.miner_id.priv_key,
            &self.miner_id.message,
        )
    }

    pub fn from_toml_str(s: &str) -> Result<Config, TomlError> {
        toml::from_str::<Config>(s)
    }

    pub fn to_toml_bytes(self) -> Vec<u8> {
        self.to_toml_string().into_bytes()
    }

    pub fn existing_config() -> bool {
        Path::new("Config.toml").exists()
    }

    pub fn read_from_toml() -> Result<Config, TomlError> {
        let mut file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("Config.toml")
            .expect("Could not open Config.toml");

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        Config::from_toml_str(&content)
    }

    pub fn write_to_toml(config: Config) {
        let mut file = File::create("Config.toml").unwrap();
        file.write_all(&config.to_toml_bytes()).unwrap();
    }

    pub async fn fetch_polynym_address(input: &str) -> Res<String> {
        let url = format!("https://api.polynym.io/getAddress/{}", input);
        let p2pkh_address = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .json::<PolynymResponse>()
            .await?;
        Ok(p2pkh_address.address)
    }

    fn optional_setup() -> Res<Config> {
        match asky::Confirm::new("Would you like to set up miner ID?").prompt()? {
            true => Prompt::run_setup(),
            false => Ok(Config::default()),
        }
    }
}

pub fn init() -> Res<()> {
    let found_config = Config::existing_config();

    let run_setup = match found_config {
        true => {
            asky::Confirm::new("Found existing config. Would you like to overwrite it?").prompt()?
        }
        false => false,
    };

    if run_setup {
        Config::optional_setup()?;
    }

    Ok(())
}
