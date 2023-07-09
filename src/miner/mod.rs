use bsv::{
    Hash, P2PKHAddress, PrivateKey, Script, ScriptTemplate, SigHash, SigningHash, Transaction,
    TxIn, TxOut, ECDSA,
};
use colored::Colorize;
use serde_json::json;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::miner_config::{MinerConfig, MinerSettings, PromptType};

pub struct MagicMiner {}

pub struct MinerResult(String, PrivateKey);

#[cfg_attr(docsrs, doc(cfg(feature = "miner")))]
impl MagicMiner {
    pub fn is_21e8_out(script: &Script) -> bool {
        let script_template = ScriptTemplate::from_asm_string(
					"OP_DATA=32 21e8 OP_SIZE OP_4 OP_PICK OP_SHA256 OP_SWAP OP_SPLIT OP_DROP OP_EQUALVERIFY OP_DROP OP_CHECKSIG",
				).unwrap();
        script.is_match(&script_template)
    }

    pub fn get_tx(txid: &str) -> Transaction {
        let url = format!("https://api.whatsonchain.com/v1/bsv/main/tx/{}/hex", txid);
        let tx_hex = reqwest::blocking::get(url).unwrap().text().unwrap();
        Transaction::from_hex(&tx_hex).unwrap()
    }

    pub fn sign(sig_hash_preimage: &Vec<u8>, target: &str) -> Option<MinerResult> {
        let ephemeral_key = PrivateKey::from_random();

        let signature = ECDSA::sign_with_deterministic_k(
            &ephemeral_key,
            &sig_hash_preimage,
            bsv::SigningHash::Sha256d,
            false,
        )
        .unwrap();

        let sighash_flag: Vec<u8> = vec![65]; // 0x41 SigHash::InputsOutputs

        let mut hashbuf = signature.to_der_bytes();
        hashbuf.extend(&sighash_flag);

        let sig256 = Hash::sha_256(&hashbuf).to_hex();

        if sig256.starts_with(target) {
            println!("\r{}", sig256.green());
            return Some(MinerResult(sig256, ephemeral_key));
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
        sig_hash_preimage: &'outer Vec<u8>,
        target: String,
    ) -> Option<MinerResult> {
        let nthreads = rayon::current_num_threads();
        let stop = Arc::new(AtomicBool::new(false));
        let pow_result = Arc::new(parking_lot::const_mutex::<Option<MinerResult>>(None));
        //println!("{} available threads.", &nthreads);

        rayon::in_place_scope(|scope| {
            for _ in 0..nthreads {
                let borrow_preimage = sig_hash_preimage.clone();
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

        let target_output = match input_tx.get_output(output_index.clone()) {
            Some(x) => x,
            None => return,
        };

        let value = &target_output.get_satoshis();

        let target = script.split(" ").collect::<Vec<&str>>()[1];

        let mut tx_in = TxIn::default();

        let locking_script = target_output.get_script_pub_key();

        tx_in.set_satoshis(value.clone());
        tx_in.set_locking_script(&locking_script);
        tx_in.set_prev_tx_id(&input_tx.get_id_bytes().unwrap());
        tx_in.set_vout(output_index.clone().try_into().unwrap());

        tx.add_input(&tx_in);

        let p2pkh = TxOut::new(value - 300u64, &pay_to_script);

        tx.add_output(&p2pkh);

        if miner_config.miner_id.enabled {
            let miner_priv = PrivateKey::from_wif(&miner_config.miner_id.priv_key).unwrap();
            let miner_pub = miner_priv.to_public_key().unwrap();

            let sig = ECDSA::sign_with_deterministic_k(
                &miner_priv,
                &input_tx.to_bytes().unwrap(),
                SigningHash::Sha256d,
                false,
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

        let sig256 = unwrapped.0;
        let ephemeral_key = unwrapped.1;

        let mut asm = "".to_owned();

        asm.push_str(&sig256); // todo: improve this
        asm.push_str(&hex::encode(&sighash));
        asm.push_str(" ");
        asm.push_str(&ephemeral_key.to_public_key().unwrap().to_hex().unwrap());

        let prev_txid = &input.get_prev_tx_id(None);
        let unlocking_script = Script::from_asm_string(&asm).unwrap();

        let tx_in = TxIn::new(&prev_txid, 0, &unlocking_script, None);

        tx.set_input(0, &tx_in);

        println!(
            "\nSigned {} with {}\n",
            target,
            ephemeral_key.to_wif().unwrap()
        );

        println!("{}\n", &tx.to_hex().unwrap().yellow());
        println!("Final txid: {}", &tx.get_id_hex().unwrap().green());

        if miner_config.autopublish {
            // todo
        }
    }

    pub fn start() {
        let txid = MinerConfig::prompt("Target TXID", PromptType::Text);

        if txid.is_empty() {
            return;
        }

        // TODO: validate input

        let tx = MagicMiner::get_tx(&txid); // Transaction::from_hex("010000000102e537778afc4ebc76a1862bd00db05e53aa017a7dab60901e8829eaa5ac5e72050000006b483045022100a12a27ebbd79e9ed16d8effc508a5007fcd01190cff84871e18e74c3283de4b702206aa45c380b3dd6765fef517dd34008f3e6cd68f47130f93f0cb2255c5e621f6f412102241bcb1a10fe297159a82e3250165e770e5da48703bbbe7834f36e2e513bf7ddffffffff070000000000000000fdb102006a067477657463684da502424945310234a4e8ec1e52f09c155b612ebf61dd7e447a209f35830b73aa356ee544c0468d5a1f9b7e248680fd3c8869806d7e3e08e7138cef96b36e89914e498b334fc56c8d02a3060c791ff016ddaaade192291893f3dd0a648d2ac1c23f68b1f2d14284773e04ad790aed17b10506c9f23def25b4f122abdd554101a3aba4631511a802e64c46cac45e398ac8176974b7627eb6b30e95b7304a0bfe0e30ca5eb2508654152f5dafac210e78b0bb64660170c3ed2566eea9c6b66b26bb8b88f42af0ff87a4c8fb5d992f2c77388c04402906ff8609605f167c7c06159f10828082c8b36579a994a8d56a2a6171006dc886e1e13d5541cf0028840ce795a646e5d3b4be9513a1a025743a0ca73e5ab38cf21e15615711c9463e10708184a0aa2377aded61e5aa78bd796ad7542edf5d95256e80b45219bf945285081834ffafe5930a0ff4f0af87c018c6f600d498798acfd709ffc4772253469d5a9ddba59420968cadbf9cc94fd97dca55bfb3add299e453102dfdd2774f30e1b60b368200be45c7a697f83b559b4f1d2a89416b1ee5c688b9ab753073838023189680ef50072c4aac1492ff7be4600c6fa86d7e7b08a69758f8ff217c8b2401b653de76ec8e51caf2095cf3bf80340de600eede56d955b535b3e4eb47e5392944fc6fff8d3ca3c6c5ee817f40ca13d1f2b14460a159b25de1012b6df27136a8aee732684582c2b9b5e86d3716e4161e9e8b1825c7efcadff57181fb72e0b341dc008c9626b21fc81a76c4cbbc0f959e8f51d0f7e208b7a0c4959c2139e27f0741ddd781b129174da6861669dd630dfe22c8557edf27ec910c4937d45f5672fcda7acb1ad42df6e538eaf90bdd01544250af2a2a0d3cdc2c35e551f5001197c2d48f36be31d472f29685bed56865edc4771dbd6df9b720d5070abff3469595a183d58bb54fb22812e9d52f4b0000000000001976a9142478c520e119b5467987b0ef8d7c48321477024a88ac5b080000000000001976a914d38242d4e5f5a6c15e50205090e1ae981c8cc4b488ac8a530000000000001976a9144bc981c7acfd339e8f360b8fa06ba7cd7d78236a88acbc020000000000002e20c24c5df0844be58fd70dc292e8dbe698f2bc7a87eaf060f6700a05144d47b7790221e8825479a87c7f758875acbc020000000000002e20fc4d602aa16170b89bda4c70d75d89e6183b14f994516aa7b3e032c3f18cbb4e0221e8825479a87c7f758875acb08b3f00000000001976a9142e23f263227cfe76fa3d4b2fd83e8dcc24a38e9288ac00000000").unwrap();

        let mut index = None;
        let outputs = tx.get_noutputs();
        let mut target_script: Option<Script> = None;

        for i in 0..outputs {
            target_script = match tx.get_output(i) {
                Some(v) => Some(v.get_script_pub_key()),
                None => continue,
            };

            if target_script.is_some() {
                if MagicMiner::is_21e8_out(&target_script.as_ref().unwrap()) {
                    index = Some(i);
                    break;
                }
            }
        }

        if index.is_none() {
            println!("No 21e8 scripts found.");
            return;
        };

        let miner_config = match MinerConfig::get_config() {
            Ok(config) => config,
            Err(e) => {
                println!("\nInvalid miner config.");
                MinerConfig::setup().unwrap()
            }
        };

        let mut to_address: String = miner_config.pay_to.clone();
        let mut p2pkh_script: Script = Script::default();

        // todo: rip polynym

        loop {
            if to_address.is_empty() {
                to_address = MinerConfig::prompt(
                    "Pay solved puzzles out to P2PKH address",
                    PromptType::Text,
                );
            }

            println!("Paying to {}", &to_address);

            match P2PKHAddress::from_string(&to_address) {
                Ok(address) => {
                    p2pkh_script = address.get_locking_script().unwrap();
                    break;
                }
                Err(e) => {
                    println!("{}\n", e);
                    return;
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
