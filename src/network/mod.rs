pub mod sandbox;
pub mod remote;

pub trait Network {
    fn passphrase() -> &'static str;
    fn send_transaction(tx: Transaction) -> Result<(), Error>;
    fn get_transaction_status(hash: Hash) -> Result<TransactionStatus, Error>;
    // ...
}

pub struct Error {}
pub struct Transaction {}
pub struct Hash {}

pub struct TransactionStatus {
}

