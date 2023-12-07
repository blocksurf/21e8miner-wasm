use crate::{MagicMiner, MinerConfig};
use inquire::Select;

pub struct CLI;

impl CLI {
    pub fn start() {
        Self::menu(vec!["Start", "Setup"]);
    }

    pub fn menu(items: Vec<&str>) {
        match Select::new("Welcome to Magic Miner", items).prompt() {
            Ok(action) => match action {
                "Setup" => MinerConfig::start(),
                "Start" => MagicMiner::start(),
                _ => (),
            },
            Err(e) => println!("{:?}", e),
        }
    }
}
