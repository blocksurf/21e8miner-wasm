# 21e8miner-wasm
Mine 21e8 faster, or don't, whatever.

# Goals
- improve performance
- add helper functions 
- multithreading
- WASM modules for the browser

## Installation

1) Install [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

2) Run the following command from the root directory:

```bash
cargo build --release
```

The binaries will be located in `<project dir>/target/release`.

### Start Miner:

```bash
cargo run --bin start
```

### Configure Miner ID and default payout settings:

```bash
cargo run --bin setup
```

## TODO:
- [x] Setup Miner ID
- [x] multithreading
- [ ] Mine from cli
- [ ] WASM modules

### What is 21e8?

Check the blockchain.

### Thanks
- Dean Little [(21e8miner)](https://github.com/deanmlittle/21e8miner)
- Nick Carton [(bsv-wasm)](https://github.com/Firaenix/bsv-wasm)
- [21e8](https://21e8.nz/)
