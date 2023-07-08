#[cfg(test)]
mod tests {
    use magic_miner::config::MinerConfig;

    #[test]
    fn get_config() {
        let config = MinerConfig::get_config();
        assert!(config.is_ok())
    }
}
