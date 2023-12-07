use std::{fs::OpenOptions, io::Write, sync::OnceLock};

use crate::Res;

fn hex_lookup() -> &'static [bool; 256] {
    static HEX_LOOKUP: OnceLock<[bool; 256]> = OnceLock::new();
    HEX_LOOKUP.get_or_init(|| {
        let mut lookup = [false; 256];
        for &c in b"0123456789ABCDEFabcdef" {
            lookup[c as usize] = true;
        }
        lookup
    })
}

pub fn is_valid_txid(txid: &str) -> bool {
    if txid.len() != 64 {
        println!("not 64 char");
        return false;
    }
    txid.bytes().all(|byte| hex_lookup()[byte as usize])
}

pub fn find_next_suffix(name: &str, folder_path: &str) -> usize {
    let mut suffix = 1;

    if std::path::Path::new(&format!("{}/{}.txt", folder_path, name)).exists() {
        return suffix;
    }

    while std::path::Path::new(&format!("{}/{}_{}.txt", folder_path, name, suffix)).exists() {
        suffix += 1;
    }
    suffix
}

pub fn write_to_file(txid: &str, raw_tx: &str) -> Res<()> {
    let folder_path = "solved";

    if !std::path::Path::new(folder_path).exists() {
        std::fs::create_dir(folder_path)?;
    }

    let suffix = find_next_suffix(txid, folder_path);

    let file_path = if suffix == 0 {
        format!("{}/{}.txt", folder_path, txid)
    } else {
        format!("{}/{}_{}.txt", folder_path, txid, suffix)
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)?;

    file.write_all(raw_tx.as_bytes())?;

    Ok(())
}
