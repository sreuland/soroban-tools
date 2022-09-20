use crate::backend::{Backend, Error, Hash, Transaction, TransactionStatus};

pub struct Remote {
    url: String,
}

impl Backend for Remote {
    fn passphrase() -> &'static str {
        panic!("not implemented");
    }

    fn send_transaction(_tx: Transaction) -> Result<(), Error> {
        panic!("not implemented");
    }

    fn get_transaction_status(_hash: Hash) -> Result<TransactionStatus, Error> {
        panic!("not implemented");
    }

    // ...
}
