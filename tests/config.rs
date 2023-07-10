#[cfg(test)]
mod tests {
    use magic_miner::config::*;

    #[test]
    fn config_exists() {
        let exists = MinerConfig::existing_config();

        println!("{}", exists)
    }

    #[test]
    fn start_setup() {
        MinerConfig::start()
    }
}
