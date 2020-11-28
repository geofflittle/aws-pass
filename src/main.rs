#[macro_use]
mod macros;
mod client;
mod creds;
mod dao;
mod store;
mod util;
use rusoto_core::Region;
use std::{env, path};
use store::default_pass_store::DefaultPassStore;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Command {
    /// Initializes the store.
    Init {},
    /// Lists the passwords given an optional **prefix**.
    List {
        #[structopt(short, long)]
        prefix: Option<String>,
    },
    /// Shows a password given a **name**.
    Show {
        #[structopt(short, long)]
        name: String,
    },
    /// Inserts a password given a **name**.
    Insert {
        #[structopt(short, long)]
        name: String,
    },
    /// Edits a password given its **name**.
    Edit {
        #[structopt(short, long)]
        name: String,
    },
    /// Generates a password given a **name**.
    Generate {
        #[structopt(short, long)]
        name: String,
    },
    /// Removes a password given its **name**.
    Remove {
        #[structopt(short, long)]
        name: String,
    },
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let store_dir = env::var("PASSWORD_STORE_DIR")
        .map(|psd| path::PathBuf::from(psd))
        .unwrap_or(dirs::home_dir().unwrap().join(".aws-pass"));
    let opt: Opt = Opt::from_args();
    let pass_store = DefaultPassStore::new(store_dir, &Region::UsEast1);
    match opt.cmd {
        Command::Init {} => pass_store.init().await,
        Command::List { prefix } => pass_store.list(prefix.as_deref()).await,
        Command::Show { name } => pass_store.show(&name).await,
        Command::Insert { name } => pass_store.insert(&name).await,
        Command::Edit { name } => pass_store.edit(&name).await,
        Command::Generate { name } => pass_store.generate(&name).await,
        Command::Remove { name } => pass_store.remove(&name).await,
    }
}
