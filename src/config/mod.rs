use bsv::PrivateKey;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::io::{Read, Write};
use toml::de::Error as TomlError;

pub struct MinerConfig {}

#[derive(Debug, Serialize, Deserialize)]
struct MinerIDSettings {
    enabled: bool,
    priv_key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinerSettings {
    miner_id: MinerIDSettings,
    pay_to: String,
    autopublish: bool,
}

pub enum PromptType {
    Text,
    Bool,
}

pub type PromptEntry = (String, PromptType);

#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
impl MinerConfig {
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

    fn generate_config_file() -> File {
        println!("Generating default config.toml...\n");

        let mut file = File::create("config.toml").expect("Create new file");

        let default = format!(
            concat!(
                "# Pay solved puzzles out to P2PKH address\n",
                "pay_to = \"\"\n",
                "# Automatically publish solved puzzles\n",
                "autopublish = false\n",
                "\n[miner_id]\n",
                "# Enable Miner API\n",
                "enabled = false\n",
                "# Private key in WIF format\n",
                "priv_key = \"{}\"\n",
                "# Select a message for Miner API\n",
                "message = \"\"\n"
            ),
            PrivateKey::from_random().to_wif().unwrap()
        );

        file.write(default.as_bytes())
            .expect("Write defaults to config.toml");

        println!("Successfully created config.toml in the root directory.\n");

        file
    }

    pub fn open_or_create_config_file() -> File {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open("config.toml")
        {
            Ok(file) => file,
            Err(ref e) if e.kind() == ErrorKind::NotFound => {
                println!("\nCould not find config.toml\n");
                MinerConfig::generate_config_file()
            }
            Err(e) => panic!("Error: {}\n", e),
        }
    }

    pub fn get_value_from_prompt(prompt: &str, prompt_type: PromptType) -> String {
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
            input = "true".to_string()
        } else if input == "n" {
            input = "false".to_string()
        }

        input
    }

    pub fn config_prompt() {
        let enabled = MinerConfig::get_value_from_prompt("Enable Miner API?", PromptType::Bool);
        let priv_key =
            MinerConfig::get_value_from_prompt("Private key in WIF format", PromptType::Text);

        let message =
            MinerConfig::get_value_from_prompt("Select a message for Miner API", PromptType::Text);

        let pay_to =
            MinerConfig::get_value_from_prompt("Pay solved puzzles out to P2PKH", PromptType::Text);

        let autopublish = MinerConfig::get_value_from_prompt(
            "Automatically publish solved puzzles?",
            PromptType::Bool,
        );

        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("config.toml")
            .expect("Opening config.toml");

        let settings = format!(
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
                "message = \"{}\"\n"
            ),
            pay_to, autopublish, enabled, priv_key, message
        );

        config_file
            .write(settings.as_bytes())
            .expect("Write defaults to config.toml");

        println!("Successfully created config.toml in the root directory.\n");
    }
}
