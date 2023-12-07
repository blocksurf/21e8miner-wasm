use crate::config::Config;
use crate::Res;
use bsv::PrivateKey;

pub struct Prompt;

impl Prompt {
    pub fn run_setup() -> Res<Config> {
        let enabled = asky::Confirm::new("Enable Miner API?").prompt()?;

        let mut priv_key: String;

        loop {
            priv_key = asky::Password::new(
                "Private key in WIF format (or press Enter to generate a new one)",
            )
            .prompt()?;

            if priv_key.is_empty() {
                priv_key = PrivateKey::from_random().to_wif()?;
                break;
            }

            match PrivateKey::from_wif(&priv_key) {
                Ok(_) => break,
                Err(e) => {
                    println!("{}\n", e);
                    continue;
                }
            }
        }

        let message = asky::Text::new("Select a message for Miner API").prompt()?;

        let mut pay_to: String;

        loop {
            pay_to = asky::Text::new(
                "Pay solved puzzle out to (1handle, $handle, PayMail or p2pkh address)",
            )
            .prompt()?;

            if !pay_to.is_empty() {
                break;
            }
        }

        let autopublish = asky::Confirm::new("Automatically publish solved puzzles?").prompt()?;

        let autosave =
            asky::Confirm::new("Automatically write solved puzzles to a .txt file?").prompt()?;

        let settings = Config::new(pay_to, autopublish, autosave, enabled, priv_key, message);

        Config::write_to_toml(settings.clone());

        Ok(settings)
    }
}
