# 21e8miner-wasm
Mine 21e8 faster, or don't, whatever.

# Goals
- improve performance
- add helper functions 
- multithreading
- WASM modules for the browser

# :warning: Disclaimer

If you want full compatibility with the OG miner, ***disable*** Miner ID. Otherwise you're going to get different signatures even with identical inputs.


#### [bsv.js](https://github.com/moneybutton/bsv) `ECDSA` assumes all hashbufs are big endian unless you set `endian: "little"`


```js
ECDSA.sign(hashbuf: Buffer, privkey: PrivateKey, endian?: 'little' | 'big')
```
That flag is required for reading TXIDs & other transaction data:

>Note that in bitcoin, the hashBuf is little endian, so if you are signing or verifying something that has to do with a transaction, you should explicitly plug in that it is little endian as an option to the sign and verify functions.

The original miner doesn't set that flag for the miner ID signature and produces the wrong `(r, s)`.

Keeping this section up until I open & merge a PR for this issue.


## TODO:
- [ ] Setup Miner ID
- [x] multithreading
- [ ] Resolve paymail addresses
- [ ] Mine from cli
- [ ] WASM modules

### What is 21e8?

Check the blockchain.

### Thanks
- Dean Little [(21e8miner)](https://github.com/deanmlittle/21e8miner)
- Nick Carton [(bsv-wasm)](https://github.com/Firaenix/bsv-wasm)
- [21e8](https://21e8.nz/)
