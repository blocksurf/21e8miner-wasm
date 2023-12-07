# 21e8miner-wasm
Mine 21e8 faster, or don't, whatever.

# Goals
- improve performance
- add helper functions 
- multithreading
- WASM modules for the browser

## Compiling

First, you'll want to check out this repository

```
git clone https://github.com/blocksurf/21e8miner-wasm.git
cd 21e8miner-wasm
```

With [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) already installed, you can run:

```bash
cargo build --release
```

The binaries will be located in `<project dir>/target/release`

## Running 21e8miner

### Start Miner:

```bash
./target/release/start
```

### Configure Miner ID and default payout settings:

```bash
./target/release/setup
```

## Publish 21e8 Jobs

Use this TX template with a certain output

```
<sha256 hash of something you want PoW for> <21e8 + target string in hex> OP_SIZE OP_4 OP_PICK OP_SHA256 OP_SWAP OP_SPLIT OP_DROP OP_EQUALVERIFY OP_DROP OP_CHECKSIG>
```

Here's a jsfiddle to get you started: https://jsfiddle.net/fkt7qb15/


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
