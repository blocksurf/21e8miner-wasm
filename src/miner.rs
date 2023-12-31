use crate::prompt::Prompt;
use crate::utils;
use crate::Config;
use asky::Text;
use bsv::{
    Hash, MatchToken, OpCodes, P2PKHAddress, PrivateKey, Script, ScriptBit, ScriptTemplate,
    SigHash, SighashSignature, Transaction, TxIn, TxOut, ECDSA,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

pub type Res<T> = anyhow::Result<T>;

pub struct MagicMiner;

pub struct MinerResult(SighashSignature, PrivateKey);

// ANSI escape
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const PURPLE: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const RESET_COLOR: &str = "\x1B[0m";

#[cfg_attr(docsrs, doc(cfg(feature = "miner")))]
impl MagicMiner {
    pub fn is_21e8(target_bit: ScriptBit) -> bool {
        if let Some(target) = target_bit.to_vec() {
            target[0] == 33 && target[1] == 232
        } else {
            false
        }
    }

    pub fn is_21e8_out(script: &Script) -> Res<bool> {
        let script_template: ScriptTemplate = ScriptTemplate::from_match_tokens(vec![
            MatchToken::Data(32, bsv::DataLengthConstraints::Equals),
            MatchToken::Data(2, bsv::DataLengthConstraints::GreaterThanOrEquals),
            MatchToken::OpCode(OpCodes::OP_SIZE),
            MatchToken::OpCode(OpCodes::OP_4),
            MatchToken::OpCode(OpCodes::OP_PICK),
            MatchToken::OpCode(OpCodes::OP_SHA256),
            MatchToken::OpCode(OpCodes::OP_SWAP),
            MatchToken::OpCode(OpCodes::OP_SPLIT),
            MatchToken::OpCode(OpCodes::OP_DROP),
            MatchToken::OpCode(OpCodes::OP_EQUALVERIFY),
            MatchToken::OpCode(OpCodes::OP_DROP),
            MatchToken::OpCode(OpCodes::OP_CHECKSIG),
        ])?;

        let target = script.get_script_bit(1).unwrap();

        Ok(script.is_match(&script_template) && MagicMiner::is_21e8(target))
    }

    pub async fn get_tx(txid: &str) -> Res<Transaction> {
        let url = format!("https://api.whatsonchain.com/v1/bsv/main/tx/{}/hex", txid);
        let tx_hex = reqwest::Client::new().get(url).send().await?.text().await?;
        Ok(Transaction::from_hex(&tx_hex)?)
    }

    pub async fn broadcast_tx(tx: &str) -> Res<String> {
        let mut map = HashMap::new();
        map.insert("txhex", tx);

        Ok(reqwest::Client::new()
            .post("https://api.whatsonchain.com/v1/bsv/main/tx/raw")
            .json(&map)
            .send()
            .await?
            .text()
            .await?)
    }

    /// 🪄
    pub fn sign(
        sender: Sender<MinerResult>,
        stop_signal: &AtomicBool,
        sig_hash_preimage: Arc<Vec<u8>>,
        target: Arc<Vec<u8>>,
    ) {
        let preimage_ref: Vec<u8> = sig_hash_preimage.iter().cloned().collect();
        let target_ref: Vec<u8> = target.iter().cloned().collect();

        loop {
            if stop_signal.load(Ordering::Relaxed) {
                return;
            }

            let ephemeral_key = PrivateKey::from_random();

            let sig = ECDSA::sign_with_deterministic_k(
                &ephemeral_key,
                &preimage_ref,
                bsv::SigningHash::Sha256d,
                false,
            )
            .unwrap();

            let sighash_signature =
                SighashSignature::new(&sig, SigHash::InputsOutputs, &preimage_ref);

            let sig256 = Hash::sha_256(&sighash_signature.to_bytes().unwrap()).to_bytes();

            if sig256.starts_with(&target_ref) {
                stop_signal.store(true, Ordering::Relaxed);

                std::thread::sleep(Duration::from_millis(100));

                println!("\r🪄 {GREEN}{}", hex::encode(sig256));
                sender
                    .send(MinerResult(sighash_signature, ephemeral_key))
                    .unwrap();

                return;
            } else if !stop_signal.load(Ordering::Relaxed) {
                print!("\r{RED}{}", hex::encode(sig256));
            }
        }
    }

    /// This is where we set up our multithreading
    pub fn mine_target(sig_hash_preimage: &[u8], target: &[u8]) -> Res<MinerResult> {
        let available_threads = std::thread::available_parallelism()?.get();
        println!("{CYAN}[{} threads]{RESET_COLOR}", available_threads);
        println!();

        let (sender, receiver) = mpsc::channel::<MinerResult>();

        let stop_signal = Arc::new(AtomicBool::new(false));

        let mut handles = Vec::with_capacity(available_threads);

        let preimage_arc = Arc::new(sig_hash_preimage.to_vec());
        let target_arc = Arc::new(target.to_vec());

        for _ in 0..available_threads {
            let sender_clone = sender.clone();
            let stop_signal_clone = Arc::clone(&stop_signal);
            let preimage_clone = Arc::clone(&preimage_arc);
            let target_clone = Arc::clone(&target_arc);

            let handle = std::thread::spawn(move || {
                MagicMiner::sign(
                    sender_clone,
                    &stop_signal_clone,
                    preimage_clone,
                    target_clone,
                );
            });

            handles.push(handle);
        }

        let result = match receiver.recv() {
            Ok(v) => v,
            Err(err) => return Err(anyhow::format_err!("recv error: {:?}", err)),
        };

        stop_signal.store(true, Ordering::Relaxed);

        for handle in handles {
            if let Err(err) = handle.join() {
                return Err(anyhow::format_err!("thread join error: {:?}", err));
            }
        }

        Ok(result)
    }

    pub async fn solve_puzzle(
        from: Transaction,
        output_index: usize,
        target: &[u8],
        pay_to_script: Script,
        miner_config: Config,
    ) -> Res<()> {
        let mut tx = Transaction::new(1, 0);

        let target_output = match from.get_output(output_index) {
            Some(x) => x,
            None => return Ok(()),
        };

        let value = &target_output.get_satoshis();

        let mut tx_in = TxIn::default();

        let locking_script = target_output.get_script_pub_key();

        tx_in.set_satoshis(*value);
        tx_in.set_locking_script(&locking_script);
        tx_in.set_prev_tx_id(&from.get_id_bytes()?);
        tx_in.set_vout(output_index as u32);

        tx.add_input(&tx_in);

        let minerid_fee = match miner_config.miner_id.enabled {
            true => 300u64,
            false => 218u64,
        };

        let p2pkh = TxOut::new(value - minerid_fee, &pay_to_script);

        tx.add_output(&p2pkh);

        if miner_config.miner_id.enabled {
            let miner_priv = PrivateKey::from_wif(&miner_config.miner_id.priv_key)?;
            let miner_pub = miner_priv.to_public_key()?;

            let sig = ECDSA::sign_digest_with_deterministic_k(&miner_priv, &from.get_id_bytes()?)?;

            let schema = json!({
                "id": miner_pub.to_hex()?,
                "sig": sig.to_der_hex(),
                "message": &miner_config.miner_id.message
            });

            let schema_bytes = schema.to_string().into_bytes();

            let encoded_pushdata = Script::encode_pushdata(&schema_bytes)?;

            let safe_data_output = Script::from_chunks(vec![encoded_pushdata])?;

            // manually build OP_RETURN script

            let op_0 = vec![0]; // OP_0
            let op_return = vec![106]; // OP_RETURN

            let op_return_script =
                Script::from_chunks(vec![op_0, op_return, safe_data_output.to_bytes()])?;

            tx.add_output(&TxOut::new(0u64, &op_return_script));
        }

        let input = tx.get_input(0).unwrap();

        let locking_script = input.get_locking_script().unwrap();

        let sats = input.get_satoshis().unwrap();

        if !MagicMiner::is_21e8_out(&locking_script)? {
            return Ok(());
        }

        let sig_hash_preimage =
            tx.sighash_preimage(SigHash::InputsOutputs, 0, &locking_script, sats)?;

        let MinerResult(sig, ephemeral_key) = MagicMiner::mine_target(&sig_hash_preimage, target)?;

        print!("{RESET_COLOR}");

        let public_key = &ephemeral_key.to_public_key()?;

        let mut unlocking_script = Script::default();

        unlocking_script.push(ScriptBit::Push(sig.to_bytes()?));
        unlocking_script.push(ScriptBit::Push(public_key.to_bytes()?));

        let prev_txid = &input.get_prev_tx_id(None);

        let tx_in_final = TxIn::new(prev_txid, output_index as u32, &unlocking_script, None);

        tx.set_input(0, &tx_in_final);

        println!(
            "\nSigned {GREEN}{}{RESET_COLOR} with {}\n",
            hex::encode(target),
            ephemeral_key.to_wif()?
        );

        let tx_hex = tx.to_hex()?;

        println!("{}{}{}\n", YELLOW, &tx_hex, RESET_COLOR);

        if miner_config.autopublish {
            let response = MagicMiner::broadcast_tx(&tx_hex).await?;
            println!("Success! {response}");
        }

        if miner_config.autosave {
            utils::write_to_file(&from.get_id_hex()?, &tx_hex)?;
        }

        Ok(())
    }

    pub async fn start() -> Res<()> {
        let txid = Text::new("Target TXID").prompt()?;

        if txid.is_empty() || !utils::is_valid_txid(&txid) {
            println!("Invalid txid");
            return Ok(());
        }

        let tx = MagicMiner::get_tx(&txid).await?;

        let mut index = None;
        let outputs = tx.get_noutputs();
        let mut target_script: Script = Script::default();

        for i in 0..outputs {
            target_script = match tx.get_output(i) {
                Some(output) => output.get_script_pub_key(),
                None => continue,
            };

            if MagicMiner::is_21e8_out(&target_script)? {
                index = Some(i);
                break;
            }
        }

        let target = match index.is_some() {
            true => target_script.get_script_bit(1).unwrap().to_vec().unwrap(),
            false => {
                println!("No 21e8 scripts found.");
                return Ok(());
            }
        };

        let miner_config = match Config::read_from_toml() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("\nInvalid miner config.\n{}", e);
                Prompt::run_setup()?
            }
        };

        let mut to_address: String = miner_config.pay_to.clone();
        let p2pkh_script: Script;

        while to_address.is_empty() {
            to_address =
                Text::new("Pay solved puzzle out to (1handle, $handle, PayMail or p2pkh address)")
                    .prompt()?;
        }

        loop {
            match P2PKHAddress::from_string(&to_address) {
                Ok(address) => {
                    p2pkh_script = address.get_locking_script()?;
                    break;
                }
                Err(e) => {
                    println!("{}\n", e);

                    // try polynym
                    to_address = match Config::fetch_polynym_address(&to_address).await {
                        Ok(v) => {
                            println!("Polynym address found: {}", v);
                            v
                        }
                        Err(e) => {
                            println!("Could not fetch address from Polynym: {:?}", e);
                            Text::new("Pay solved puzzle out to (P2PKH address)").prompt()?
                        }
                    };
                }
            };
        }

        println!(
            "{GREEN}■{RESET_COLOR} Paying to: {PURPLE}{}{RESET_COLOR}",
            &to_address
        );
        print!("{GREEN}■{RESET_COLOR} Mining output {} ", &index.unwrap());

        MagicMiner::solve_puzzle(tx, index.unwrap(), &target, p2pkh_script, miner_config).await
    }
}
