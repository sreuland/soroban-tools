use std::array::TryFromSliceError;
use std::num::ParseIntError;
use std::{fmt::Debug, fs, io};

use clap::Parser;
use soroban_env_host::xdr::{
    Error as XdrError, Hash, HostFunction, InstallContractCodeArgs, InvokeHostFunctionOp,
    LedgerFootprint, LedgerKey::ContractCode, LedgerKeyContractCode, Memo, MuxedAccount, Operation,
    OperationBody, Preconditions, SequenceNumber, Transaction, TransactionEnvelope, TransactionExt,
    Uint256, VecM,
};
use soroban_env_host::HostError;

use crate::rpc::{self, Client};
use crate::snapshot::{self, get_default_ledger_info};
use crate::{utils, HEADING_RPC, HEADING_SANDBOX};

#[derive(Parser, Debug)]
pub struct Cmd {
    /// WASM file to install
    #[clap(long, parse(from_os_str))]
    wasm: std::path::PathBuf,
    /// File to persist ledger state
    #[clap(
        long,
        parse(from_os_str),
        default_value = ".soroban/ledger.json",
        conflicts_with = "rpc-url",
        env = "SOROBAN_LEDGER_FILE",
        help_heading = HEADING_SANDBOX,
    )]
    ledger_file: std::path::PathBuf,

    /// Secret 'S' key used to sign the transaction sent to the rpc server
    #[clap(
        long = "secret-key",
        env = "SOROBAN_SECRET_KEY",
        help_heading = HEADING_RPC,
    )]
    secret_key: Option<String>,
    /// RPC server endpoint
    #[clap(
        long,
        requires = "secret-key",
        requires = "network-passphrase",
        env = "SOROBAN_RPC_URL",
        help_heading = HEADING_RPC,
    )]
    rpc_url: Option<String>,
    /// Network passphrase to sign the transaction sent to the rpc server
    #[clap(
        long = "network-passphrase",
        env = "SOROBAN_NETWORK_PASSPHRASE",
        help_heading = HEADING_RPC,
    )]
    network_passphrase: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("error parsing int: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("internal conversion error: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("xdr processing error: {0}")]
    Xdr(#[from] XdrError),
    #[error("jsonrpc error: {0}")]
    JsonRpc(#[from] jsonrpsee_core::Error),
    #[error("reading file {filepath}: {error}")]
    CannotReadLedgerFile {
        filepath: std::path::PathBuf,
        error: snapshot::Error,
    },
    #[error("reading file {filepath}: {error}")]
    CannotReadContractFile {
        filepath: std::path::PathBuf,
        error: io::Error,
    },
    #[error("committing file {filepath}: {error}")]
    CannotCommitLedgerFile {
        filepath: std::path::PathBuf,
        error: snapshot::Error,
    },
    #[error("cannot parse secret key")]
    CannotParseSecretKey,
    #[error(transparent)]
    Rpc(#[from] rpc::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let contract = fs::read(&self.wasm).map_err(|e| Error::CannotReadContractFile {
            filepath: self.wasm.clone(),
            error: e,
        })?;

        let res_str = if self.rpc_url.is_some() {
            self.run_against_rpc_server(contract).await?
        } else {
            self.run_in_sandbox(contract)?
        };
        println!("{res_str}");
        Ok(())
    }

    fn run_in_sandbox(&self, contract: Vec<u8>) -> Result<String, Error> {
        let mut state =
            snapshot::read(&self.ledger_file).map_err(|e| Error::CannotReadLedgerFile {
                filepath: self.ledger_file.clone(),
                error: e,
            })?;
        let wasm_hash = utils::add_contract_code_to_ledger_entries(&mut state.1, contract)?;

        snapshot::commit(state.1, get_default_ledger_info(), [], &self.ledger_file).map_err(
            |e| Error::CannotCommitLedgerFile {
                filepath: self.ledger_file.clone(),
                error: e,
            },
        )?;
        Ok(hex::encode(wasm_hash))
    }

    async fn run_against_rpc_server(&self, contract: Vec<u8>) -> Result<String, Error> {
        let client = Client::new(self.rpc_url.as_ref().unwrap());
        let key = utils::parse_secret_key(self.secret_key.as_ref().unwrap())
            .map_err(|_| Error::CannotParseSecretKey)?;

        // Get the account sequence number
        let public_strkey =
            stellar_strkey::StrkeyPublicKeyEd25519(key.public.to_bytes()).to_string();
        let account_details = client.get_account(&public_strkey).await?;
        // TODO: create a cmdline parameter for the fee instead of simply using the minimum fee
        let fee: u32 = 100;
        let sequence = account_details.sequence.parse::<i64>()?;

        let (tx, hash) = build_install_contract_code_tx(
            contract,
            sequence + 1,
            fee,
            self.network_passphrase.as_ref().unwrap(),
            &key,
        )?;
        client.send_transaction(&tx).await?;

        Ok(hex::encode(hash.0))
    }
}

pub(crate) fn build_install_contract_code_tx(
    contract: Vec<u8>,
    sequence: i64,
    fee: u32,
    network_passphrase: &str,
    key: &ed25519_dalek::Keypair,
) -> Result<(TransactionEnvelope, Hash), XdrError> {
    let hash = utils::contract_hash(&contract)?;

    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            function: HostFunction::InstallContractCode(InstallContractCodeArgs {
                code: contract.try_into()?,
            }),
            footprint: LedgerFootprint {
                read_only: VecM::default(),
                read_write: vec![ContractCode(LedgerKeyContractCode { hash: hash.clone() })]
                    .try_into()?,
            },
        }),
    };

    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(key.public.to_bytes())),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![op].try_into()?,
        ext: TransactionExt::V0,
    };

    let envelope = utils::sign_transaction(key, &tx, network_passphrase)?;

    Ok((envelope, hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_install_contract_code() {
        let result = build_install_contract_code_tx(
            b"foo".to_vec(),
            300,
            1,
            "Public Global Stellar Network ; September 2015",
            &utils::parse_secret_key("SBFGFF27Y64ZUGFAIG5AMJGQODZZKV2YQKAVUUN4HNE24XZXD2OEUVUP")
                .unwrap(),
        );

        assert!(result.is_ok());
    }
}
