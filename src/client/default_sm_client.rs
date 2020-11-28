use super::sm_client::{Filter, SecretDetails, SecretString, SecretsPage, SmClient, SmClientErr, Tag};
use async_trait::async_trait;
use log::info;
use rusoto_core::{credential, HttpClient, Region};
use rusoto_secretsmanager::{
    CreateSecretRequest, DeleteSecretRequest, DescribeSecretRequest, GetSecretValueRequest, ListSecretsRequest,
    PutSecretValueRequest, SecretsManager, SecretsManagerClient,
};

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
                HttpClient::new().expect("Unable to new HttpClient"),
                provide_aws_creds,
                region.clone(),
            )),
        }
    }
}

#[async_trait]
impl SmClient for DefaultSmClient {
    async fn create_secret_string(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String, SmClientErr> {
        let create_secret_request = CreateSecretRequest {
            client_request_token: Some(uuid::Uuid::new_v4().to_string()),
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
        Ok(create_secret_response
            .expect("Unable to get create secret response")
            .arn
            .expect("Unable to get arn"))
    }

    async fn delete_secret(&self, arn: &str) -> Result<(), SmClientErr> {
        let delete_secret_request = DeleteSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send delete secret request {:?}", delete_secret_request);
        let delete_secret_response = self.sm_client.delete_secret(delete_secret_request).await;
        info!("Did receive delete secret response {:?}", delete_secret_response);
        delete_secret_response.expect("Unable to get delete secret response");
        Ok(())
    }

    async fn delete_secret_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<(), SmClientErr> {
        let secret = self
            .get_secret_string_by_name(name, filters)
            .await
            .expect("Unable to get secret string by name");
        self.delete_secret(&secret.arn).await.expect("Unable to delete secret");
        Ok(())
    }

    async fn describe_secret(&self, arn: &str) -> Result<SecretDetails, SmClientErr> {
        let describe_secret_request = DescribeSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send describe secret request {:?}", describe_secret_request);
        let describe_secret_response = self.sm_client.describe_secret(describe_secret_request).await;
        info!("Did receive describe secret result {:?}", describe_secret_response);
        Ok(describe_secret_response.expect("Unable to get secret response")).map(|s| SecretDetails {
            arn: s.arn.expect("Unable to get secret arn"),
            name: s.name.expect("Unable to get secret name"),
            tags: translate_tags(s.tags),
            description: s.description,
        })
    }

    async fn get_secret_string(&self, arn: &str) -> Result<SecretString, SmClientErr> {
        let get_secret_value_request = GetSecretValueRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        info!("Will send get secret value request {:?}", get_secret_value_request);
        let get_secret_value_response = self.sm_client.get_secret_value(get_secret_value_request).await;
        info!("Did receive get secret value response {:?}", get_secret_value_response);
        Ok(get_secret_value_response.expect("Unable to get get secret value response")).map(|s| SecretString {
            arn: s.arn.expect("Unable to get secret arn"),
            name: s.name.expect("Unable to get secret name"),
            value: s.secret_string.expect("Unable to get secret string"),
        })
    }

    async fn get_secret_string_by_name(
        &self,
        name: &str,
        filters: Option<&[Filter]>,
    ) -> Result<SecretString, SmClientErr> {
        let all_filters: &[Filter] = &[
            &vec![("name".to_string(), vec![name.to_string()])],
            filters.unwrap_or_default(),
        ]
        .concat();
        let secrets = self
            .list_all_secrets(Some(&all_filters))
            .await
            .expect("Unable to list all secrets");
        Ok(self
            .get_secret_string(&secrets.iter().next().expect("Unable to get secret").arn)
            .await
            .expect("Unable to get secret string"))
        .map(|s| SecretString {
            arn: s.arn,
            name: s.name,
            value: s.value,
        })
    }

    async fn list_secrets(
        &self,
        filters: Option<&[Filter]>,
        next_token: Option<&str>,
    ) -> Result<SecretsPage, SmClientErr> {
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
        Ok(list_secrets_response.expect("Unable to get list secrets response")).map(|lsr| {
            (
                lsr.secret_list.map_or(Vec::new(), |sl| {
                    sl.into_iter()
                        .map(|s| SecretDetails {
                            arn: s.arn.expect("Unable to get secret arn"),
                            name: s.name.expect("Unable to get secret name"),
                            tags: translate_tags(s.tags),
                            description: s.description,
                        })
                        .collect()
                }),
                lsr.next_token,
            )
        })
    }

    async fn list_all_secrets(&self, filters: Option<&[Filter]>) -> Result<Vec<SecretDetails>, SmClientErr> {
        let mut vec: Vec<SecretDetails> = Vec::new();
        // Couldn't get Option<&str> to work
        let mut next_token: Option<String> = None;
        loop {
            // TODO: Use better error handling
            let mut page = self
                .list_secrets(filters, next_token.as_deref())
                .await
                .expect("Unable to list secrets");
            vec.append(&mut page.0);
            next_token = page.1;
            if next_token.is_none() {
                break;
            }
        }
        Ok(vec)
    }

    async fn put_secret_string(&self, arn: &str, value: &str) -> Result<(), SmClientErr> {
        let put_secret_value_request = PutSecretValueRequest {
            secret_id: arn.to_string(),
            secret_string: Some(value.to_string()),
            ..Default::default()
        };
        info!("Will send put secret value request {:?}", put_secret_value_request);
        let put_secret_value_response = self.sm_client.put_secret_value(put_secret_value_request).await;
        info!("Did receive put secret value response {:?}", put_secret_value_response);
        put_secret_value_response.expect("Unable to get put secret value response");
        Ok(())
    }

    async fn put_secret_string_by_name(
        &self,
        name: &str,
        value: &str,
        filters: Option<&[Filter]>,
    ) -> Result<(), SmClientErr> {
        let secret = self
            .get_secret_string_by_name(name, filters)
            .await
            .expect("Unable to get secret string by name");
        self.put_secret_string(&secret.arn, value).await
    }
}

fn translate_tags(tags: Option<Vec<rusoto_secretsmanager::Tag>>) -> Vec<(String, String)> {
    tags.map_or(Vec::new(), |tags| {
        tags.into_iter()
            .map(|tag| {
                (
                    tag.key.expect("Unable to get tag key"),
                    tag.value.expect("Unable to get tag value"),
                )
            })
            .collect()
    })
}
