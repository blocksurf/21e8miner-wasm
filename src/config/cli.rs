use crate::{MagicMiner, Res};
use inquire::Select;

pub struct CLI;

impl CLI {
    pub async fn menu(items: Vec<&str>) -> Res<()> {
        match Select::new("Welcome to Magic Miner", items).prompt() {
            Ok(action) => match action {
                "Setup" => crate::config::start()?,
                "Start" => MagicMiner::start().await?,
                _ => (),
            },
            Err(e) => println!("{:?}", e),
        };

        Ok(())
    }
}

pub async fn start() -> Res<()> {
    CLI::menu(vec!["Start", "Setup"]).await
}
