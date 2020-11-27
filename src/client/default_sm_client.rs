use super::sm_client::{
    Filter, SMClient, SMClientErr, SecretDetails, SecretString, SecretsPage,
};
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_secretsmanager::{
    CreateSecretRequest, DeleteSecretRequest, DescribeSecretRequest,
    GetSecretValueRequest, ListSecretsRequest, PutSecretValueRequest,
    SecretsManager, SecretsManagerClient, Tag,
};

pub struct DefaultSMClient {
    sm_client: Box<dyn SecretsManager + Sync + Send>,
}

impl DefaultSMClient {
    pub fn new(region: Region) -> impl SMClient {
        DefaultSMClient {
            sm_client: Box::new(SecretsManagerClient::new(region)),
        }
    }
}

#[async_trait]
impl SMClient for DefaultSMClient {
    async fn create_secret_string(
        &self,
        name: &str,
        value: &str,
    ) -> Result<String, SMClientErr> {
        let create_secret_request = CreateSecretRequest {
            name: name.to_string(),
            secret_string: Some(value.to_string()),
            ..Default::default()
        };
        // TODO: Use better error handling
        Ok(self
            .sm_client
            .create_secret(create_secret_request)
            .await
            .unwrap()
            .arn
            .unwrap())
    }

    async fn delete_secret(&self, arn: &str) -> Result<(), SMClientErr> {
        let delete_secret_request = DeleteSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        // TODO: Use better error handling
        self.sm_client
            .delete_secret(delete_secret_request)
            .await
            .unwrap();
        Ok(())
    }

    async fn describe_secret(
        &self,
        arn: &str,
    ) -> Result<SecretDetails, SMClientErr> {
        let describe_secret_request = DescribeSecretRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        // TODO: Use better error handling
        let describe_secret_response = self
            .sm_client
            .describe_secret(describe_secret_request)
            .await
            .unwrap();
        Ok(SecretDetails {
            arn: describe_secret_response.arn.unwrap(),
            name: describe_secret_response.name.unwrap(),
            tags: translate_tags(describe_secret_response.tags),
            description: describe_secret_response.description,
        })
    }

    async fn get_secret_string(
        &self,
        arn: &str,
    ) -> Result<SecretString, SMClientErr> {
        let get_secret_value_request = GetSecretValueRequest {
            secret_id: arn.to_string(),
            ..Default::default()
        };
        // TODO: Use better error handling
        let get_secret_value_response = self
            .sm_client
            .get_secret_value(get_secret_value_request)
            .await
            .unwrap();
        Ok(SecretString {
            arn: get_secret_value_response.arn.unwrap(),
            name: get_secret_value_response.name.unwrap(),
            value: get_secret_value_response.secret_string.unwrap(),
        })
    }

    async fn get_secret_string_by_name(
        &self,
        name: &str,
        filters: Option<&[Filter]>,
    ) -> Result<SecretString, SMClientErr> {
        let all_filters: Vec<Filter> = [
            &vec![("name".to_string(), vec![name.to_string()])],
            filters.unwrap_or_default(),
        ]
        .concat();
        let next_token: Option<String> = None;
        loop {
            // TODO: Use better error handling
            let page = self
                .list_secrets(Some(&all_filters), next_token.as_deref())
                .await
                .unwrap();
            if let Some(secret) = page.0.into_iter().find(|s| s.name == name) {
                return Ok(self.get_secret_string(&secret.arn).await.unwrap());
            }
            if next_token.is_none() {
                break;
            }
        }
        Err(SMClientErr::SecretNotFound)
    }

    async fn list_secrets(
        &self,
        filters: Option<&[Filter]>,
        next_token: Option<&str>,
    ) -> Result<SecretsPage, SMClientErr> {
        let list_secrets_request = ListSecretsRequest {
            filters: filters.map(|fs| {
                fs.into_iter()
                    .map(|(k, vs)| rusoto_secretsmanager::Filter {
                        key: Some(k.to_string()),
                        values: Some(
                            vs.into_iter().map(|v| v.to_string()).collect(),
                        ),
                    })
                    .collect()
            }),
            next_token: next_token.map(|nt| nt.to_string()),
            ..Default::default()
        };
        // TODO: Use better error handling
        let list_secrets_response = self
            .sm_client
            .list_secrets(list_secrets_request)
            .await
            .unwrap();
        Ok((
            list_secrets_response.secret_list.map_or(Vec::new(), |sl| {
                sl.into_iter()
                    .map(|sle| SecretDetails {
                        arn: sle.arn.unwrap(),
                        name: sle.name.unwrap(),
                        tags: translate_tags(sle.tags),
                        description: sle.description,
                    })
                    .collect()
            }),
            list_secrets_response.next_token,
        ))
    }

    async fn list_all_secrets(
        &self,
        filters: Option<&[Filter]>,
    ) -> Result<Vec<SecretDetails>, SMClientErr> {
        let mut vec: Vec<SecretDetails> = Vec::new();
        // Couldn't get Option<&str> to work
        let mut next_token: Option<String> = None;
        loop {
            // TODO: Use better error handling
            let mut page = self
                .list_secrets(filters, next_token.as_deref())
                .await
                .unwrap();
            vec.append(&mut page.0);
            next_token = page.1;
            if next_token.is_none() {
                break;
            }
        }
        Ok(vec)
    }

    async fn put_secret_string(
        &self,
        arn: &str,
        value: &str,
    ) -> Result<(), SMClientErr> {
        let put_secret_request = PutSecretValueRequest {
            secret_id: arn.to_string(),
            secret_string: Some(value.to_string()),
            ..Default::default()
        };
        // TODO: Use better error handling
        self.sm_client
            .put_secret_value(put_secret_request)
            .await
            .unwrap();
        Ok(())
    }

    async fn put_secret_string_by_name(
        &self,
        name: &str,
        value: &str,
        filters: Option<&[Filter]>,
    ) -> Result<(), SMClientErr> {
        // TODO: Use better error handling
        let secret =
            self.get_secret_string_by_name(name, filters).await.unwrap();
        self.put_secret_string(&secret.arn, value).await
    }
}

fn translate_tags(tags: Option<Vec<Tag>>) -> Vec<(String, String)> {
    tags.map_or(Vec::new(), |tags| {
        tags.into_iter()
            .map(|tag| (tag.key.unwrap(), tag.value.unwrap()))
            .collect()
    })
}
