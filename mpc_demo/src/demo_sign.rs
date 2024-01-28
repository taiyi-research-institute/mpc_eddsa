pub const MSG: &str =
    "Je ne veux pas travailler. Je ne veux pas déjeuner. Je veux seulement l'oublier. Et puis je fume.";

#[tokio::main] // `tokio` re-exported by `mpc_sesman::prelude::*`
async fn main() -> Outcome<()> {
    let matches = Command::new("demo_keygen")
        .arg(
            Arg::new("signer_id")
                .short('i')
                .required(true)
                .value_parser(value_parser!(u16))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("signers")
                .short('s')
                .required(true)
                .value_parser(value_parser!(u16))
                .num_args(1..)
                .value_delimiter(' ')
                .action(ArgAction::Set),
        )
        .get_matches();

    let signer_id = *matches.get_one::<u16>("signer_id").ifnone_()?;
    let signers: HashSet<u16> = matches
        .get_many::<u16>("signers")
        .ifnone_()?
        .cloned()
        .collect();
    println!("signer_id: {}, signers: {:?}", signer_id, &signers);

    let keystore: KeyStore = {
        let path = &format!("assets/{}@demo_keygen.keystore", signer_id);
        let mut file = File::open(path).await.catch_()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.catch_()?;
        let keystore = serde_json::from_slice(&buf).catch_()?;
        keystore
    };
    let msg_hash: Vec<u8> = {
        let mut hasher = Sha512::new();
        hasher.update(MSG.as_bytes());
        hasher.finalize().to_vec()
    };

    let sig = algo_sign(&signers, "m/1/14/514", &msg_hash, &keystore)
        .await
        .catch_()?;
    '_print: {
        let sig_r = hex::encode(&sig.r.compress().as_bytes());
        let sig_s = hex::encode(&sig.z.as_bytes());
        let tx_hash = hex::encode(&sig.hash);
        println!("sig_r: {}", sig_r);
        println!("sig_s: {}", sig_s);
        println!("tx_hash: {}", tx_hash);
    }

    Ok(())
}

use std::collections::HashSet;

use clap::{value_parser, Arg, ArgAction, Command};
use mpc_algo::{algo_sign, KeyStore};
use mpc_sesman::prelude::tokio::{fs::File, io::AsyncReadExt};
use mpc_sesman::prelude::*;
use sha2::{Digest, Sha512};
