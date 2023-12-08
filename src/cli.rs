use crate::{MagicMiner, Res};
use asky::{Select, SelectOption};

pub struct CLI;

const HEADER: &str =
    "┌┬┐┌─┐┌─┐┬┌─┐\n│││├─┤│ ┬││  \n┴ ┴┴ ┴└─┘┴└─┘\n┌┬┐┬┌┐┌┌─┐┬─┐\n│││││││├┤ ├┬┘\n┴ ┴┴┘└┘└─┘┴└─\n";

impl CLI {
    pub async fn menu<'a>(items: Vec<SelectOption<'a, &'a str>>) -> Res<()> {
        println!("{HEADER}");

        match Select::new_complex("⛏️ ", items).prompt() {
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
    CLI::menu(vec![SelectOption::new("Start"), SelectOption::new("Setup")]).await
}
