use bsv::PrivateKey;
use std::io::Write;

use crate::config::MinerConfig;
pub struct Prompt {}

#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
impl Prompt {
    pub fn read_input() -> String {
        std::io::stdout().flush().unwrap();

        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        if input.ends_with('\n') {
            input.pop();
            if input.ends_with('\r') {
                input.pop();
            }
        };

        input
    }

    pub fn confirm_prompt(prompt: &str) -> bool {
        print!("{} (y/n): ", prompt);

        let input = Prompt::read_input();

        if input == "y" {
            return true;
        };

        false
    }

    pub fn text_prompt(prompt: &str) -> String {
        print!("{}: ", prompt);
        Prompt::read_input()
    }

    pub fn run_setup() -> MinerConfig {
        let enabled = Prompt::confirm_prompt("Enable Miner API?");

        let mut priv_key: String;

        loop {
            priv_key = Prompt::text_prompt(
                "Private key in WIF format (or press Enter to generate a new one)",
            );

            if priv_key.is_empty() {
                priv_key = PrivateKey::from_random().to_wif().unwrap();
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

        let message = Prompt::text_prompt("Select a message for Miner API");

        let mut pay_to: String;

        loop {
            pay_to = Prompt::text_prompt(
                "Pay solved puzzle out to (1handle, $handle, PayMail or p2pkh address)",
            );

            if !pay_to.is_empty() {
                break;
            }
        }

        let autopublish = Prompt::confirm_prompt("Automatically publish solved puzzles?");

        let settings = MinerConfig::new(pay_to, autopublish, enabled, priv_key, message);

        MinerConfig::write_to_toml(settings.clone());

        settings
    }
}
