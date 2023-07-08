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
}
