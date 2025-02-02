use clap::Parser;
use soroban_env_host::meta;
use std::fmt::Debug;

const GIT_REVISION: &str = env!("GIT_REVISION");

#[derive(Parser, Debug)]
pub struct Cmd;

impl Cmd {
    #[allow(clippy::unused_self)]
    pub fn run(&self) {
        println!("soroban {} ({})", env!("CARGO_PKG_VERSION"), GIT_REVISION);

        let env = soroban_env_host::VERSION;
        println!("soroban-env {} ({})", env.pkg, env.rev);
        println!("soroban-env interface version {}", meta::INTERFACE_VERSION);

        let xdr = soroban_env_host::VERSION.xdr;
        println!(
            "stellar-xdr {} ({})
xdr next ({})",
            xdr.pkg, xdr.rev, xdr.xdr_next,
        );
    }
}

// Check that the XDR cannel in use is 'next' to ensure that the version output
// is not forgotten when we eventually update to using curr. This is a bit of a
// hack because of limits of what you can do in a constant context, but by being
// a constant context this is checked at compile time.
const _: () = {
    #[allow(clippy::single_match)]
    match soroban_env_host::VERSION.xdr.xdr.as_bytes() {
        b"next" => (),
        _ => panic!("xdr version channel needs updating"),
    }
};
