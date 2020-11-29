use super::pass_store::PassStore;
use crate::{
    creds::StsLocalMfaCredsProvider,
    dao::{
        pass_dao::Tag,
        pass_dao::{Filter, PassDao, Password},
        sm_pass_dao::SmPassDao,
    },
    util,
};
use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::Region;
use std::fs;
use std::{path::PathBuf, process};
use util::write_lines;

const CREDENTIALS_FILENAME: &str = ".credentials";
const TOKEN_SERIAL_FILENAME: &str = ".token-serial";
// TODO: Fix tags
const STORE_TAGS: (&str, &str) = ("aws-pass", "true");
const STORE_FILTERS: [(&str, [&str; 1]); 2] = [("tag-key", ["aws-pass"]), ("tag-value", ["true"])];

struct StoreDetails {
    access_key_id: String,
    secret_access_key: String,
    token_serial: String,
}

pub struct DefaultPassStore {
    store_dir: PathBuf,
    pass_dao: Box<dyn PassDao + Send + Sync>,
}

impl DefaultPassStore {
    pub fn new(store_dir: PathBuf, region: &Region) -> Box<dyn PassStore> {
        let creds_provider = StsLocalMfaCredsProvider::new(
            store_dir.join(CREDENTIALS_FILENAME),
            store_dir.join(TOKEN_SERIAL_FILENAME),
            region,
        );
        Box::new(DefaultPassStore {
            store_dir,
            pass_dao: Box::new(SmPassDao::new(creds_provider, region)),
        })
    }

    fn ensure_empty_store_dir(&self) {
        if self.store_dir.exists() && self.store_dir.is_dir() && self.store_dir.read_dir().unwrap().next().is_some() {
            fatal_println!("Store dir {} not empty, not overwriting", self.store_dir.display())
        }
        if self.store_dir.exists() && self.store_dir.is_file() {
            fatal_println!("Store dir {} not a directory", self.store_dir.display())
        }
        if !self.store_dir.exists() {
            println!("Creating store dir at {}", self.store_dir.display());
            fs::create_dir(&self.store_dir).unwrap();
        }
    }

    fn get_store_details(&self) -> StoreDetails {
        let access_key_id = util::prompt_non_empty_str("AWS Access Key Id");
        let secret_access_key = util::prompt_non_empty_str("AWS Secret Access Key");
        let token_serial = util::prompt_non_empty_str("MFA Token Serial Number");
        StoreDetails {
            access_key_id,
            secret_access_key,
            token_serial,
        }
    }

    fn write_store_details(
        &self,
        StoreDetails {
            access_key_id,
            secret_access_key,
            token_serial,
        }: &StoreDetails,
    ) {
        let creds_path = self.store_dir.join(CREDENTIALS_FILENAME);
        write_lines(
            &creds_path,
            vec![
                "[default]\n",
                format!("aws_access_key_id={}\n", access_key_id).as_ref(),
                format!("aws_secret_access_key={}\n", secret_access_key).as_ref(),
            ],
        );

        let token_serial_path = self.store_dir.join(TOKEN_SERIAL_FILENAME);
        write_lines(&token_serial_path, vec![token_serial.as_ref()]);
    }

    async fn get_password_by_name(&self, name: &str) -> Result<Password> {
        let filters: Vec<Filter> = STORE_FILTERS
            .iter()
            .map(|f| (f.0.to_string(), f.1.iter().map(|s| s.to_string()).collect()))
            .collect();
        self.pass_dao.get_password_by_name(name, Some(&filters)).await
    }
}

#[async_trait]
impl PassStore for DefaultPassStore {
    async fn init(&self) {
        self.ensure_empty_store_dir();
        println!(
            "Please provide AWS credentials for a user with an associated policy with MFA-protected SecretsManager \
            permissions"
        );
        let creds = self.get_store_details();
        self.write_store_details(&creds);
    }

    async fn list(&self, prefix: Option<&str>) {
        let ssfilters = STORE_FILTERS
            .iter()
            .map(|f| (f.0.to_string(), f.1.iter().map(|s| s.to_string()).collect()))
            .collect();
        let filters: Vec<Filter> = [
            ssfilters,
            prefix
                .map(|p| vec![("name".to_string(), vec![p.to_string()])])
                .unwrap_or_default(),
        ]
        .concat();
        let passwords = self.pass_dao.list_passwords(&filters).await.unwrap();
        let names: Vec<String> = passwords.into_iter().map(|p| p.name).collect();
        println!("{}", names.join("\n"));
    }

    async fn show(&self, name: &str) {
        let password = self.get_password_by_name(name).await.unwrap();
        println!("{}", password.value);
    }

    async fn insert(&self, name: &str) {
        let value = util::prompt_stdin_line("Enter password:");
        let tags: Vec<Tag> = vec![(STORE_TAGS.0.to_string(), STORE_TAGS.1.to_string())];
        self.pass_dao.create_password(name, &value, Some(&tags)).await.unwrap();
    }

    async fn edit(&self, name: &str) {
        let password = self.get_password_by_name(name).await.unwrap();
        let updated_password = edit::edit(password.value).unwrap().trim_end().to_string();
        self.pass_dao
            .update_password(&password.id, &updated_password)
            .await
            .unwrap();
    }

    async fn generate(&self, name: &str, exclude_chars: Option<&str>, length: Option<&i64>) {
        let tags: Vec<Tag> = vec![(STORE_TAGS.0.to_string(), STORE_TAGS.1.to_string())];
        let password = self
            .pass_dao
            .create_random_password(name, exclude_chars, length, Some(&tags))
            .await
            .unwrap();
        println!("{}", password.value);
    }

    async fn remove(&self, name: &str) {
        let filters: Vec<Filter> = STORE_FILTERS
            .iter()
            .map(|f| (f.0.to_string(), f.1.iter().map(|s| s.to_string()).collect()))
            .collect();
        self.pass_dao
            .delete_password_by_name(name, Some(&filters))
            .await
            .unwrap();
    }
}
