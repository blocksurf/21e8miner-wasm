use bsv::{P2PKHAddress, PrivateKey};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::Path;
use toml::de::Error as TomlError;

pub struct MinerConfig {}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinerIDSettings {
    pub enabled: bool,
    pub priv_key: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinerSettings {
    pub miner_id: MinerIDSettings,
    pub pay_to: String,
    pub autopublish: bool,
}

pub enum PromptType {
    Text,
    Bool,
}

pub type PromptEntry = (String, PromptType);

#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
impl MinerConfig {
    fn toml_string(
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

    pub fn get_config() -> Result<MinerSettings, TomlError> {
        let mut file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("config.toml")
            .expect("Could not open config.toml");

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        toml::from_str::<MinerSettings>(&content)
    }

    fn default() -> File {
        println!("Generating default config.toml...\n");

        let mut file = File::create("config.toml").expect("Create new file");

        let default = MinerConfig::toml_string(
            "",
            "true",
            "false",
            &PrivateKey::from_random().to_wif().unwrap(),
            "",
        );

        file.write(default.as_bytes())
            .expect("Write defaults to config.toml");

        println!("Successfully created config.toml in the root directory.\n");

        file
    }

    pub fn open_or_default() -> File {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open("config.toml")
        {
            Ok(file) => file,
            Err(ref e) if e.kind() == ErrorKind::NotFound => {
                println!("\nCould not find config.toml\n");
                MinerConfig::default()
            }
            Err(e) => panic!("Error: {}\n", e),
        }
    }

    pub fn prompt(prompt: &str, prompt_type: PromptType) -> String {
        match prompt_type {
            PromptType::Text => println!("{}", prompt),
            PromptType::Bool => println!("{} (y/n)", prompt),
        }

        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        }

        if input == "y" {
            input = "true".into()
        } else if input == "n" {
            input = "false".into()
        }

        input
    }

    pub fn setup() {
        let enabled = MinerConfig::prompt("Enable Miner API?", PromptType::Bool);

        let mut priv_key: String;

        loop {
            priv_key = MinerConfig::prompt(
                "Private key in WIF format (or press Enter to generate a new one)",
                PromptType::Text,
            );

            if priv_key.is_empty() {
                priv_key = PrivateKey::from_random().to_wif().unwrap();
                break;
            }

            match PrivateKey::from_wif(&priv_key) {
                Ok(_) => break,
                Err(e) => {
                    println!("{}\n", e);
                }
            }
        }

        let message = MinerConfig::prompt("Select a message for Miner API", PromptType::Text);

        let mut pay_to: String;

        loop {
            pay_to = MinerConfig::prompt("Pay solved puzzles out to P2PKH", PromptType::Text);

            match P2PKHAddress::from_string(&pay_to) {
                Ok(_) => break,
                Err(e) => {
                    println!("{}\n", e);
                }
            }
        }

        let autopublish =
            MinerConfig::prompt("Automatically publish solved puzzles?", PromptType::Bool);

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("config.toml")
            .expect("Opening config.toml");

        let settings =
            MinerConfig::toml_string(&pay_to, &autopublish, &enabled, &priv_key, &message);

        config_file
            .write(settings.as_bytes())
            .expect("Write defaults to config.toml");

        println!("Successfully created config.toml in the root directory.\n");
    }

    pub fn existing_config() -> bool {
        Path::new("config.toml").exists()
    }

    pub fn init() {
        match MinerConfig::existing_config() {
            true => {
                let config = MinerConfig::get_config().unwrap();
                println!("Config found!\n\n{:#?}", config)
            }
            false => MinerConfig::setup(),
        }
    }
}
