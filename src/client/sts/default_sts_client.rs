use super::sts_client::{Credentials, StsClient};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{NaiveDateTime, TimeZone, Utc};
use log::info;
use rusoto_core::{credential, HttpClient, Region};
use rusoto_sts::{GetSessionTokenRequest, Sts};

pub struct DefaultStsClient {
    sts_client: Box<dyn Sts + Send + Sync>,
}

impl DefaultStsClient {
    pub fn new<P>(provide_aws_creds: P, region: &Region) -> impl StsClient
    where
        P: credential::ProvideAwsCredentials + Send + Sync + 'static,
    {
        DefaultStsClient {
            sts_client: Box::new(rusoto_sts::StsClient::new_with(
                HttpClient::new().unwrap(),
                provide_aws_creds,
                region.clone(),
            )),
        }
    }
}

#[async_trait]
impl StsClient for DefaultStsClient {
    async fn get_session_token(
        &self,
        duration_seconds: Option<&i64>,
        serial_number: Option<&str>,
        token_code: Option<&str>,
    ) -> Result<Credentials> {
        let get_session_token_request = GetSessionTokenRequest {
            duration_seconds: duration_seconds.map(|i| i.to_owned()),
            serial_number: serial_number.map(String::from),
            token_code: token_code.map(String::from),
        };
        info!("Will send get session token request {:?}", get_session_token_request);
        let get_session_token_response = self.sts_client.get_session_token(get_session_token_request).await;
        info!("Did receive session token response {:?}", get_session_token_response);
        get_session_token_response?
            .credentials
            .map(|c| {
                Ok(Credentials {
                    access_key_id: c.access_key_id,
                    secret_access_key: c.secret_access_key,
                    session_token: c.session_token,
                    expiration: Utc
                        .from_utc_datetime(&NaiveDateTime::parse_from_str(&c.expiration, "%Y-%m-%dT%H:%M:%S%Z")?),
                })
            })
            .unwrap()
    }
}
