# 21e8miner-wasm
Mine 21e8 faster, or don't, whatever.

# Goals
- improve performance
- add helper functions 
- multithreading
- WASM modules for the browser

# :warning: Disclaimer

There are some compatibility issues between [bsv.js](https://github.com/moneybutton/bsv) and [bsv-wasm](https://github.com/Firaenix/bsv-wasm) when generating the Miner ID signature.

If you want full compatibility with the OG miner, ***disable*** Miner ID. Otherwise you're going to get different signatures even with identical inputs.


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
