extern crate bsv;

pub mod miner;
pub use miner::*;

pub mod config;
pub use crate::config as miner_config;
pub use miner_config::*;

pub fn start() {
    MagicMiner::start()
}

pub fn setup() {
    MinerConfig::start()
}
