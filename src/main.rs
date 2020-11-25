extern crate pretty_env_logger;
extern crate rusoto_core;
extern crate rusoto_secretsmanager;

mod subcommands;
mod traits;

use clap::Clap;
use traits::AsyncRunnable;

#[derive(Clap)]
enum SubCmd {
    /// Initializes the store
    #[clap(version = "1.0")]
    Init(subcommands::init::Init),
    /// Lists the passwords
    #[clap(version = "1.0")]
    List(subcommands::list::List),
    /// Shows a password
    #[clap(version = "1.0")]
    Show(subcommands::show::Show),
    /// Inserts a password
    #[clap(version = "1.0")]
    Insert(subcommands::insert::Insert),
}

/// A password manager that stores passwords in AWS SecretsManager.
#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCmd,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let opts: Opts = Opts::parse();
    match opts.sub_cmd {
        SubCmd::Init(init) => init.run().await,
        SubCmd::List(list) => list.run().await,
        SubCmd::Show(show) => show.run().await,
        SubCmd::Insert(insert) => insert.run().await,
    };
    Ok(())
}
