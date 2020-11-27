extern crate pretty_env_logger;
extern crate rusoto_core;
extern crate rusoto_secretsmanager;
mod client;
mod dao;
mod store;
mod util;
use dao::{
    local_pass_dao::LocalPassDao, pass_dao::PassDao, sm_pass_dao::SMPassDao,
};
use rusoto_core::Region;
use std::{env, path};
use store::default_pass_store::DefaultPassStore;
use store::pass_store::PassStore;
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
    #[structopt(short, long)]
    local: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[tokio::main]
async fn main() {
    let store_dir = env::var("PASSWORD_STORE_DIR")
        .map(|psd| path::PathBuf::from(psd))
        .unwrap_or(dirs::home_dir().unwrap().join(".aws-pass"));

    let opt: Opt = Opt::from_args();
    let pass_dao: Box<dyn PassDao + Send + Sync> = {
        match opt.local {
            true => Box::new(LocalPassDao::new()),
            false => Box::new(SMPassDao::new(Region::UsEast1)),
        }
    };
    let pass_store: Box<dyn PassStore> =
        DefaultPassStore::new(store_dir, pass_dao);
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
