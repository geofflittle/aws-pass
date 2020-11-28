use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusoto_core::credential::AwsCredentials;

#[async_trait]
pub trait StsClient {
    async fn get_session_token(
        &self,
        duration_seconds: Option<&i64>,
        serial_numer: Option<&str>,
        token_code: Option<&str>,
    ) -> Result<Credentials, StsClientErr>;
}

#[derive(Debug)]
pub enum StsClientErr {}

pub trait Creds {
    fn is_expired(&self) -> bool;
    fn to_aws_creds(&self) -> AwsCredentials;
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: DateTime<Utc>,
}

impl Creds for Credentials {
    fn is_expired(&self) -> bool {
        self.expiration.le(&chrono::Utc::now())
    }

    fn to_aws_creds(&self) -> AwsCredentials {
        AwsCredentials::new(
            self.access_key_id.clone(),
            self.secret_access_key.clone(),
            Some(self.session_token.clone()),
            Some(self.expiration.clone()),
        )
    }
}

// impl Default for Credentials {
//     fn default() -> Self {
//         Credentials {
//             access_key_id: "".to_string(),
//             secret_access_key: "".to_string(),
//             session_token: "".to_string(),
//             expiration: Utc::now(),
//         }
//     }
// }
