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
    sync::Mutex,
};

pub struct StsLocalMfaCredsProvider {
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
        let serial_number = read_first_line(&self.token_serial_path).expect(&format!(
            "Token path {} has no first line",
            self.token_serial_path.display()
        ));
        let token_code = prompt_stdin_line("MFA token code:");
        self.sts_client
            .get_session_token(Some(&900), Some(&serial_number), Some(&token_code))
            .await
            .expect("No session token")
    }
}

#[async_trait]
impl credential::ProvideAwsCredentials for StsLocalMfaCredsProvider {
    async fn credentials(&self) -> Result<credential::AwsCredentials, credential::CredentialsError> {
        // match self.cached_creds.is_expired() {
        //     false => {
        //         return Ok(self.cached_creds.to_aws_creds());
        //     }
        //     true => {
        //         let new_creds = self.get_creds().await;
        //         *self.cached_creds = new_creds;
        //         return Ok(new_creds.to_aws_creds());
        //     }
        // }
        //
        //
        // let mutex = self.cached_creds.try_read().expect("Unable to get creds lock");
        // match mutex.is_expired() {
        //     false => {
        //         return Ok(mutex.to_aws_creds());
        //     }
        //     true => {
        //         let new_creds = self.get_creds().await;
        //         *mutex = Box::new(new_creds);
        //         return Ok(new_creds.to_aws_creds());
        //     }
        // }
        Ok(self.get_creds().await.to_aws_creds())
    }
}
