use std::path::PathBuf;
use crate::backend::{Backend, Error, Hash, Transaction, TransactionStatus};

pub static PASSPHRASE: &str = "Local Sandbox Stellar Network ; September 2022";

pub struct Sandbox {
    dir: PathBuf,
}

impl Network for Sandbox {
    fn passphrase() -> &'static str {
        PASSPHRASE
    }

    fn send_transaction(_tx: Transaction) -> Result<(), Error> {
        panic!("not implemented");
    }

    fn get_transaction_status(_hash: Hash) -> Result<TransactionStatus, Error> {
        panic!("not implemented");
    }

    // ...
}
