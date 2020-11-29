use super::sm_client::{Filter, SecretDetails, SecretString, SecretsPage, SmClient, Tag};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use rusoto_core::{credential, HttpClient, Region};
use rusoto_secretsmanager::{
    CreateSecretRequest, DeleteSecretRequest, DescribeSecretRequest, GetRandomPasswordRequest, GetSecretValueRequest,
    ListSecretsRequest, PutSecretValueRequest, SecretsManager, SecretsManagerClient,
};
use uuid::Uuid;

pub struct DefaultSmClient {
    sm_client: Box<dyn SecretsManager + Send + Sync>,
}

impl DefaultSmClient {
    pub fn new<P>(provide_aws_creds: P, region: &Region) -> impl SmClient
    where
        P: credential::ProvideAwsCredentials + Send + Sync + 'static,
    {
        DefaultSmClient {
            sm_client: Box::new(SecretsManagerClient::new_with(
                HttpClient::new().unwrap(),
                provide_aws_creds,
                region.clone(),
            )),
        }
    }
}

#[async_trait]
impl SmClient for DefaultSmClient {
    async fn create_secret_string(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String> {
        let create_secret_request = CreateSecretRequest {
            client_request_token: Some(Uuid::new_v4().to_string()),
            name: name.to_string(),
            secret_string: Some(value.to_string()),
            tags: tags.map(|ts| {
                ts.into_iter()
                    .map(|t| rusoto_secretsmanager::Tag {
                        key: Some(t.0.clone()),
                        value: Some(t.1.clone()),
                    })
                    .collect()
            }),
            ..Default::default()
        };
        info!("Will send create secret request {:?}", create_secret_request);
        let create_secret_response = self.sm_client.create_secret(create_secret_request).await;
        info!("Did receive create secret response {:?}", create_secret_response);
        Ok(create_secret_response?.arn.unwrap())
    }

    async fn delete_secret(&self, arn: &str) -> Result<()> {
        let delete_secret_request = DeleteSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send delete secret request {:?}", delete_secret_request);
        let delete_secret_response = self.sm_client.delete_secret(delete_secret_request).await;
        info!("Did receive delete secret response {:?}", delete_secret_response);
        Ok(delete_secret_response?).map(|_| ())
    }

    async fn describe_secret(&self, arn: &str) -> Result<SecretDetails> {
        let describe_secret_request = DescribeSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send describe secret request {:?}", describe_secret_request);
        let describe_secret_response = self.sm_client.describe_secret(describe_secret_request).await;
        info!("Did receive describe secret result {:?}", describe_secret_response);
        Ok(describe_secret_response?).map(|s| SecretDetails {
            arn: s.arn.unwrap(),
            name: s.name.unwrap(),
            tags: translate_tags(s.tags),
            description: s.description,
        })
    }

    async fn get_secret_string(&self, arn: &str) -> Result<SecretString> {
        let get_secret_value_request = GetSecretValueRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send get secret value request {:?}", get_secret_value_request);
        let get_secret_value_response = self.sm_client.get_secret_value(get_secret_value_request).await;
        info!("Did receive get secret value response {:?}", get_secret_value_response);
        Ok(get_secret_value_response?).map(|s| SecretString {
            arn: s.arn.unwrap(),
            name: s.name.unwrap(),
            value: s.secret_string.unwrap(),
        })
    }

    async fn list_secrets(&self, filters: Option<&[Filter]>, next_token: Option<&str>) -> Result<SecretsPage> {
        let list_secrets_request = ListSecretsRequest {
            filters: filters.map(|fs| {
                fs.iter()
                    .map(|(k, vs)| rusoto_secretsmanager::Filter {
                        key: Some(k.to_string()),
                        values: Some(vs.iter().map(|v| v.to_string()).collect()),
                    })
                    .collect()
            }),
            next_token: next_token.map(|nt| nt.to_string()),
            ..Default::default()
        };
        info!("Will send list secrets request {:?}", list_secrets_request);
        // TODO: Use better error handling
        let list_secrets_response = self.sm_client.list_secrets(list_secrets_request).await;
        info!("Did receive list secrets response {:?}", list_secrets_response);
        Ok(list_secrets_response?).map(|lsr| {
            (
                lsr.secret_list.map_or(Vec::new(), |sl| {
                    sl.into_iter()
                        .map(|s| SecretDetails {
                            arn: s.arn.unwrap(),
                            name: s.name.unwrap(),
                            tags: translate_tags(s.tags),
                            description: s.description,
                        })
                        .collect()
                }),
                lsr.next_token,
            )
        })
    }

    async fn put_secret_string(&self, arn: &str, value: &str) -> Result<()> {
        let put_secret_value_request = PutSecretValueRequest {
            client_request_token: Some(Uuid::new_v4().to_string()),
            secret_id: arn.to_string(),
            secret_string: Some(value.to_string()),
            ..Default::default()
        };
        info!("Will send put secret value request {:?}", put_secret_value_request);
        let put_secret_value_response = self.sm_client.put_secret_value(put_secret_value_request).await;
        info!("Did receive put secret value response {:?}", put_secret_value_response);
        Ok(put_secret_value_response?).map(|_| ())
    }

    async fn get_random_password(&self, exclude_chars: Option<&str>, length: Option<&i64>) -> Result<String> {
        let get_random_password_request = GetRandomPasswordRequest {
            exclude_characters: exclude_chars.map(|s| s.to_string()),
            password_length: length.map(|l| l.clone()),
            ..Default::default()
        };
        info!(
            "Will send get random password request {:?}",
            get_random_password_request
        );
        let get_random_password_response = self.sm_client.get_random_password(get_random_password_request).await?;
        info!(
            "Did receive get random password response {:?}",
            get_random_password_response
        );
        Ok(get_random_password_response.random_password.unwrap())
    }
}

fn translate_tags(tags: Option<Vec<rusoto_secretsmanager::Tag>>) -> Vec<(String, String)> {
    tags.map_or(Vec::new(), |tags| {
        tags.into_iter()
            .map(|tag| (tag.key.unwrap(), tag.value.unwrap()))
            .collect()
    })
}
