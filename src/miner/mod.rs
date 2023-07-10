use bsv::{
    Hash, MatchToken, OpCodes, P2PKHAddress, PrivateKey, Script, ScriptTemplate, SigHash,
    Signature, SigningHash, Transaction, TxIn, TxOut, ECDSA,
};
use colored::Colorize;
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::miner_config::{MinerConfig, MinerSettings, PromptType};

pub struct MagicMiner {}

pub struct MinerResult(Signature, PrivateKey);

#[cfg_attr(docsrs, doc(cfg(feature = "miner")))]
impl MagicMiner {
    pub fn is_21e8_out(script: &Script) -> bool {
        let script_template: ScriptTemplate = ScriptTemplate::from_match_tokens(vec![
            MatchToken::Data(32, bsv::DataLengthConstraints::Equals),
            MatchToken::Push(vec![33, 232]),
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
        ])
        .unwrap();
        script.is_match(&script_template)
    }

    pub fn get_tx(txid: &str) -> Transaction {
        let url = format!("https://api.whatsonchain.com/v1/bsv/main/tx/{}/hex", txid);
        let tx_hex = reqwest::blocking::get(url).unwrap().text().unwrap();
        Transaction::from_hex(&tx_hex).unwrap()
    }

    pub fn broadcast_tx(tx: &str) {
        let mut map = HashMap::new();
        map.insert("txhex", tx);

        let client = reqwest::blocking::Client::new();
        match client
            .post("https://api.whatsonchain.com/v1/bsv/main/tx/raw")
            .json(&map)
            .send()
        {
            Ok(res) => println!("Published! {}", res.text().unwrap()),
            Err(e) => println!("{:?}", e),
        };
    }

    pub fn sign(sig_hash_preimage: &[u8], target: &str) -> Option<MinerResult> {
        let ephemeral_key = PrivateKey::from_random();

        let signature = ECDSA::sign_with_deterministic_k(
            &ephemeral_key,
            sig_hash_preimage,
            bsv::SigningHash::Sha256d,
            false,
            false,
        )
        .unwrap();

        let sighash_flag: Vec<u8> = vec![65]; // 0x41 SigHash::InputsOutputs

        let mut hashbuf = signature.to_der_bytes();
        hashbuf.extend(&sighash_flag);

        let sig256 = Hash::sha_256(&hashbuf).to_hex();

        if sig256.starts_with(target) {
            println!("\r{}", sig256.green());
            return Some(MinerResult(signature, ephemeral_key));
        } else {
            print!("\r{}", sig256.red());
            match std::io::stdout().flush() {
                Ok(_) => print!(""),
                Err(error) => println!("{}", error),
            }
            return None;
        }
    }

    pub fn mine_parallel<'outer>(
        sig_hash_preimage: &'outer [u8],
        target: String,
    ) -> Option<MinerResult> {
        let nthreads = rayon::current_num_threads();
        let stop = Arc::new(AtomicBool::new(false));
        let pow_result = Arc::new(parking_lot::const_mutex::<Option<MinerResult>>(None));
        //println!("{} available threads.", &nthreads);

        rayon::in_place_scope(|scope| {
            for _ in 0..nthreads {
                let borrow_preimage = sig_hash_preimage.to_owned();
                let cloned_target = target.clone();
                let stop = stop.clone();
                let pow_result = pow_result.clone();

                scope.spawn(move |_| {
                    while !stop.load(Ordering::Acquire) {
                        match MagicMiner::sign(&borrow_preimage, &cloned_target) {
                            Some(v) => {
                                stop.store(true, std::sync::atomic::Ordering::Release);

                                let mut pow_result = if let Some(pow_result) = pow_result.try_lock()
                                {
                                    pow_result
                                } else {
                                    // Another thread is writing a result, this thread can break.
                                    break;
                                };

                                *pow_result = Some(v)
                            }
                            None => {
                                continue;
                            }
                        };
                    }
                })
            }
        });

        if let Ok(suffix) = Arc::try_unwrap(pow_result) {
            if let Some(result) = suffix.into_inner() {
                return Some(result);
            } else {
                return None;
            }
        } else {
            return None;
        };
    }

    pub fn mine_id(
        input_tx: Transaction,
        output_index: usize,
        script: String,
        pay_to_script: Script,
        miner_config: MinerSettings,
    ) {
        let mut tx = Transaction::new(1, 0);

        let target_output = match input_tx.get_output(output_index) {
            Some(x) => x,
            None => return,
        };

        let value = &target_output.get_satoshis();

        let target = script.split('\u{0020}').collect::<Vec<&str>>()[1];

        let mut tx_in = TxIn::default();

        let locking_script = target_output.get_script_pub_key();

        tx_in.set_satoshis(*value);
        tx_in.set_locking_script(&locking_script);
        tx_in.set_prev_tx_id(&input_tx.get_id_bytes().unwrap());
        tx_in.set_vout(output_index.try_into().unwrap());

        tx.add_input(&tx_in);

        let minerid_fee = match miner_config.miner_id.enabled {
            true => 300u64,
            false => 218u64,
        };

        let p2pkh = TxOut::new(value - minerid_fee, &pay_to_script);

        tx.add_output(&p2pkh);

        if miner_config.miner_id.enabled {
            let miner_priv = PrivateKey::from_wif(&miner_config.miner_id.priv_key).unwrap();
            let miner_pub = miner_priv.to_public_key().unwrap();

            let sig = ECDSA::sign_with_deterministic_k(
                &miner_priv,
                &input_tx.to_bytes().unwrap(),
                SigningHash::Sha256d,
                true,
                true,
            )
            .unwrap();

            let schema = json!({
                "id": miner_pub.to_hex().unwrap(),
                "sig": sig.to_der_hex(),
                "message": &miner_config.miner_id.message
            });

            let schema_bytes = schema.to_string().into_bytes();

            let encoded_pushdata = Script::encode_pushdata(&schema_bytes).unwrap();

            let safe_data_output = Script::from_chunks(vec![encoded_pushdata]).unwrap();

            // manually build OP_RETURN script

            let op_0 = vec![0]; // Script::from_script_bits(vec![ScriptBit::OpCode(OpCodes::OP_0)]).to_bytes();
            let op_return = vec![106]; //Script::from_script_bits(vec![ScriptBit::OpCode(OpCodes::OP_RETURN)]).to_bytes();

            let op_return_script =
                Script::from_chunks(vec![op_0, op_return, safe_data_output.to_bytes()]).unwrap();

            tx.add_output(&TxOut::new(0u64, &op_return_script));
        }

        let input = tx.get_input(0).unwrap();

        let locking_script = input.get_locking_script().unwrap();

        let sats = input.get_satoshis().unwrap();

        if !MagicMiner::is_21e8_out(&locking_script) {
            return;
        }

        let sig_hash_preimage = tx
            .sighash_preimage(SigHash::InputsOutputs, 0, &locking_script, sats)
            .unwrap();

        let sighash: Vec<u8> = vec![65];
        let mut pow_result: Option<MinerResult> = None;

        while pow_result.is_none() {
            pow_result = MagicMiner::mine_parallel(&sig_hash_preimage, target.to_string());
        }

        let unwrapped = pow_result.unwrap();

        let sig = unwrapped.0;
        let ephemeral_key = unwrapped.1;

        let mut asm = "".to_owned();

        let mut der = sig.to_der_bytes();
        der.append(&mut sighash.clone());

        let formatted_length = Script::get_pushdata_bytes(der.len()).unwrap();
        asm.push_str(&format!("{:x}", formatted_length[0]));

        asm.push_str(&sig.to_der_hex());
        asm.push_str(&hex::encode(&sighash));

        let public_key = &ephemeral_key.to_public_key().unwrap();

        asm.push_str(&format!("{:x}", public_key.to_bytes().unwrap().len()));
        asm.push_str(&public_key.to_hex().unwrap());

        let prev_txid = &input.get_prev_tx_id(None);

        let unlocking_script = Script::from_hex(&asm).unwrap();

        let tx_in_final = TxIn::new(
            prev_txid,
            output_index.try_into().unwrap(),
            &unlocking_script,
            None,
        );

        tx.set_input(0, &tx_in_final);

        println!(
            "\nSigned {} with {}\n",
            target,
            ephemeral_key.to_wif().unwrap()
        );

        let tx_hex = tx.to_hex().unwrap();

        println!("{}\n", &tx_hex.yellow());

        if miner_config.autopublish {
            MagicMiner::broadcast_tx(&tx_hex)
        }
    }

    pub fn start() {
        let txid = MinerConfig::prompt("Target TXID", PromptType::Text);

        if txid.is_empty() {
            return;
        }

        // TODO: validate input

        let tx = MagicMiner::get_tx(&txid);

        let mut index = None;
        let outputs = tx.get_noutputs();
        let mut target_script: Option<Script> = None;

        for i in 0..outputs {
            target_script = match tx.get_output(i) {
                Some(v) => Some(v.get_script_pub_key()),
                None => continue,
            };

            if target_script.is_some() && MagicMiner::is_21e8_out(target_script.as_ref().unwrap()) {
                index = Some(i);
                break;
            }
        }

        if index.is_none() {
            println!("No 21e8 scripts found.");
            return;
        };

        let miner_config = match MinerConfig::get_config() {
            Ok(config) => config,
            Err(e) => {
                println!("\nInvalid miner config.\n{}", e);
                MinerConfig::setup().unwrap()
            }
        };

        let mut to_address: String = miner_config.pay_to.clone();
        let p2pkh_script: Script;

        loop {
            if to_address.is_empty() {
                to_address = MinerConfig::prompt(
                    "Pay solved puzzle out to (1handle, $handle, PayMail or p2pkh address)",
                    PromptType::Text,
                );
            }

            to_address = MinerConfig::get_address(&to_address);

            println!("Paying to {}", &to_address);

            match P2PKHAddress::from_string(&to_address) {
                Ok(address) => {
                    p2pkh_script = address.get_locking_script().unwrap();
                    break;
                }
                Err(e) => {
                    println!("{}\n", e);
                    continue;
                }
            };
        }

        println!("Mining TX {} output {:?}", txid.trim(), &index.unwrap());

        MagicMiner::mine_id(
            tx,
            index.unwrap(),
            target_script.unwrap().to_asm_string(),
            p2pkh_script,
            miner_config,
        );
    }
}
