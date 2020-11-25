use crate::traits::AsyncRunnable;
use async_trait::async_trait;
use clap::Clap;

#[derive(Clap)]
pub struct List {}

#[async_trait]
impl AsyncRunnable for List {
    async fn run(self) {
        todo!()
        // pretty_env_logger::init();
        // let client = SecretsManagerClient::new(Region::UsEast1);
        // let list_secrets_request: ListSecretsRequest = Default::default();
        // let output = client
        //     .list_secrets(list_secrets_request)
        //     .await
        //     .map_err(|e| format!("{:?}", e))?;
        // output.secret_list.into_iter().for_each(|secret_entries| {
        //     secret_entries.into_iter().for_each(|secret_entry| {
        //         let name = secret_entry.name.unwrap_or("<NO NAME>".to_string());
        //         let arn = secret_entry.arn.unwrap_or("<NO ARN>".to_string());
        //         info!("Name: {}, Arn: {}", name, arn)
        //     })
        // });
    }
}
