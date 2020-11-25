use crate::traits::AsyncRunnable;
use async_trait::async_trait;
use clap::Clap;
use rusoto_core::Region;
use rusoto_secretsmanager::{Filter, ListSecretsRequest, SecretsManager, SecretsManagerClient};
use std::{
    fs,
    io::{BufRead, BufReader},
};

#[derive(Clap)]
pub struct Insert {
    /// The password name
    #[clap()]
    name: String,
}

#[async_trait]
impl AsyncRunnable for Insert {
    async fn run(self) {
        println!("begin");
        if self.name.is_empty() {
            panic!("name must be non-empty");
        }
        let id_file_path = dirs::home_dir().unwrap().join(".aws-pass").join(".id");
        if !id_file_path.exists() {
            panic!("No id file");
        }
        println!("id file");
        let id_file = fs::File::open(id_file_path).unwrap();
        let mut buffer = BufReader::new(id_file);
        let mut id = String::new();
        buffer.read_line(&mut id).unwrap();
        if id.is_empty() {
            panic!("id must be non-empty");
        }
        println!("id");
        let client = SecretsManagerClient::new(Region::UsEast1);
        let list_secrets_request = ListSecretsRequest {
            filters: Some(vec![
                Filter {
                    key: Some("aws-pass-id".to_string()),
                    values: Some(vec![id.clone()]),
                },
                Filter {
                    key: Some("aws-pass-name".to_string()),
                    values: Some(vec![self.name.clone()]),
                },
            ]),
            ..Default::default()
        };
        let output = client.list_secrets(list_secrets_request).await.unwrap();
        println!("{:?}", output);
        output.secret_list.into_iter().for_each(|secret_entries| {
            secret_entries.into_iter().for_each(|secret_entry| {
                if let Some(secret_name) = secret_entry.name {
                    if secret_name == self.name {
                        panic!("Secret with provided name already exists")
                    }
                }
            })
        })
    }
}
