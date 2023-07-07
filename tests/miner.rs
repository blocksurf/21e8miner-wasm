#[cfg(test)]
mod tests {
    use e8_miner::MagicMiner;

    #[test]
    fn mine_id() {
        let txid = String::from("daed53994962c6f3ce5803eeac51d38166a8bad7ed555a39da15f2733b7048f7");
        MagicMiner::start(Some(&txid));
    }
}
