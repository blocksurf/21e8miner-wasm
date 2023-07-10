# 21e8miner-wasm
Mine 21e8 faster, or don't, whatever.

# Goals
- improve performance
- add helper functions 
- multithreading
- WASM modules for the browser

## Installation

Make sure you've got Cargo installed then run `cargo build --release` in the root directory.

### Start Miner:

```
cargo run --bin start
```

### Configure Miner ID and default payout settings:

```
cargo run --bin setup
```

## TODO:
- [x] Setup Miner ID
- [x] multithreading
- [] Mine from cli
- [ ] WASM modules

### What is 21e8?

Check the blockchain.

### Thanks
- Dean Little [(21e8miner)](https://github.com/deanmlittle/21e8miner)
- Nick Carton [(bsv-wasm)](https://github.com/Firaenix/bsv-wasm)
- [21e8](https://21e8.nz/)
