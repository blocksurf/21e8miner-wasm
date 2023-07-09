#[cfg(test)]
mod tests {
    use magic_miner::config::MinerConfig;

    #[test]
    fn get_config() {
        let config = MinerConfig::get_config();
        assert!(config.is_ok())
    }

    #[test]
    fn open_or_create_default() {
        MinerConfig::open_or_default();
    }

    #[test]
    fn config_prompt() {
        MinerConfig::setup();
    }

    #[test]
    fn config_exists() {
        let exists = MinerConfig::existing_config();

        println!("{}", exists)
    }

    #[test]
    fn init() {
        MinerConfig::init()
    }
}
