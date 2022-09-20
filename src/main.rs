use clap::{AppSettings, CommandFactory, FromArgMatches, Parser, Subcommand};

mod commands;
mod jsonrpc;
mod network;
mod snapshot;
mod strval;
mod utils;

#[derive(Parser, Debug)]
#[clap(
    version,
    about = "https://soroban.stellar.org",
    disable_help_subcommand = true,
    disable_version_flag = true
)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Root {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Invoke a contract function in a WASM file
    Invoke(commands::invoke::Cmd),
    /// Inspect a WASM file listing contract functions, meta, etc
    Inspect(commands::inspect::Cmd),
    /// Print the current value of a contract-data ledger entry
    Read(commands::read::Cmd),
    /// Run a local webserver for web app development and testing
    Serve(commands::serve::Cmd),
    /// Deploy a WASM file as a contract
    Deploy(commands::deploy::Cmd),
    /// Generate code client bindings for a contract
    Gen(commands::gen::Cmd),

    /// Print version information
    Version(commands::version::Cmd),
    /// Print shell completion code for the specified shell.
    #[clap(long_about = commands::completion::LONG_ABOUT)]
    Completion(commands::completion::Cmd),
}

#[derive(thiserror::Error, Debug)]
enum CmdError {
    // TODO: stop using Debug for displaying errors
    #[error(transparent)]
    Inspect(#[from] commands::inspect::Error),
    #[error(transparent)]
    Invoke(#[from] commands::invoke::Error),
    #[error(transparent)]
    Read(#[from] commands::read::Error),
    #[error(transparent)]
    Serve(#[from] commands::serve::Error),
    #[error(transparent)]
    Gen(#[from] commands::gen::Error),
    #[error(transparent)]
    Deploy(#[from] commands::deploy::Error),
}

async fn run(cmd: Cmd, matches: &mut clap::ArgMatches) -> Result<(), CmdError> {
    match cmd {
        Cmd::Inspect(inspect) => inspect.run()?,
        Cmd::Invoke(invoke) => {
            let (_, sub_arg_matches) = matches.remove_subcommand().unwrap();
            invoke.run(&sub_arg_matches)?;
        }
        Cmd::Read(read) => read.run()?,
        Cmd::Serve(serve) => serve.run().await?,
        Cmd::Gen(gen) => gen.run()?,
        Cmd::Deploy(deploy) => deploy.run()?,
        Cmd::Version(version) => version.run(),
        Cmd::Completion(completion) => completion.run(&mut Root::command()),
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    // We expand the Root::parse() invocation, so that we can save
    // Clap's ArgMatches (for later argument processing)
    let mut matches = Root::command().get_matches();
    let mut saved_matches = matches.clone();
    let root = match Root::from_arg_matches_mut(&mut matches) {
        Ok(s) => s,
        Err(e) => {
            let mut cmd = Root::command();
            e.format(&mut cmd).exit()
        }
    };

    if let Err(e) = run(root.cmd, &mut saved_matches).await {
        eprintln!("error: {}", e);
    }
}
