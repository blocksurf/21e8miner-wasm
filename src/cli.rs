use crate::{MagicMiner, Res};
use asky::Select;

pub struct CLI;

const HEADER: &str =
    "┌┬┐┌─┐┌─┐┬┌─┐\n│││├─┤│ ┬││  \n┴ ┴┴ ┴└─┘┴└─┘\n┌┬┐┬┌┐┌┌─┐┬─┐\n│││││││├┤ ├┬┘\n┴ ┴┴┘└┘└─┘┴└─\n";

impl CLI {
    pub async fn menu(items: Vec<&str>) -> Res<()> {
        println!("{HEADER}");

        match Select::new("⛏️", items).prompt() {
            Ok(action) => match action {
                "Setup" => crate::config::init()?,
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
