extern crate pretty_env_logger;
extern crate rusoto_core;
extern crate rusoto_secretsmanager;

use log::info;
use rusoto_core::Region;
use rusoto_secretsmanager::{ListSecretsRequest, SecretsManager, SecretsManagerClient};

#[tokio::main]
async fn main() -> Result<(), String> {
    pretty_env_logger::init();
    let client = SecretsManagerClient::new(Region::UsEast1);
    let list_secrets_request: ListSecretsRequest = Default::default();
    let output = client
        .list_secrets(list_secrets_request)
        .await
        .map_err(|e| format!("{:?}", e))?;
    output.secret_list.into_iter().for_each(|secret_entries| {
        secret_entries.into_iter().for_each(|secret_entry| {
            let name = secret_entry.name.unwrap_or("<NO NAME>".to_string());
            let arn = secret_entry.arn.unwrap_or("<NO ARN>".to_string());
            info!("Name: {}, Arn: {}", name, arn)
        })
    });
    Ok(())
}
