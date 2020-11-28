use crate::{
    client::sts::{
        default_sts_client::DefaultStsClient,
        sts_client::{Credentials, Creds, StsClient},
    },
    util::{prompt_stdin_line, read_first_line},
};
use async_trait::async_trait;
use rusoto_core::{credential, Region};
use std::{
    path::{self, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

pub struct StsLocalMfaCredsProvider {
    cached_creds: Arc<Mutex<Option<Credentials>>>,
    token_serial_path: path::PathBuf,
    sts_client: Box<dyn StsClient + Send + Sync>,
}

impl StsLocalMfaCredsProvider {
    pub fn new(
        creds_path: PathBuf,
        token_serial_path: PathBuf,
        region: &Region,
    ) -> impl credential::ProvideAwsCredentials {
        StsLocalMfaCredsProvider {
            cached_creds: Arc::new(Mutex::new(None)),
            token_serial_path,
            sts_client: Box::new(DefaultStsClient::new(
                credential::ProfileProvider::with_default_configuration(creds_path),
                region,
            )),
        }
    }
}

impl StsLocalMfaCredsProvider {
    async fn get_creds(&self) -> Credentials {
        let serial_number = read_first_line(&self.token_serial_path).unwrap();
        let token_code = prompt_stdin_line("MFA token code:");
        self.sts_client
            .get_session_token(Some(&900), Some(&serial_number), Some(&token_code))
            .await
            .unwrap()
    }
}

#[async_trait]
impl credential::ProvideAwsCredentials for StsLocalMfaCredsProvider {
    async fn credentials(&self) -> Result<credential::AwsCredentials, credential::CredentialsError> {
        let mut mutex = self.cached_creds.lock().await;
        if mutex.is_some() && !mutex.as_ref().unwrap().is_expired() {
            return Ok(mutex.as_ref().unwrap().to_aws_creds());
        }
        let new_creds = self.get_creds().await;
        *mutex = Some(new_creds.clone());
        return Ok(new_creds.to_aws_creds());
    }
}
