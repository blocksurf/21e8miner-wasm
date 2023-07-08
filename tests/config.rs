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
        MinerConfig::open_or_create_config_file();
    }

    #[test]
    fn config_prompt() {
        MinerConfig::config_prompt();
    }
}
