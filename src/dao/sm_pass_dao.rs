use super::pass_dao::{Filter, PassDao, Password, PasswordDetails, Tag};
use crate::client::sm::{default_sm_client::DefaultSmClient, sm_client::SmClient};
use anyhow::Result;
use async_trait::async_trait;
use rusoto_core::{credential, Region};

pub struct SmPassDao {
    sm_client: Box<dyn SmClient + Send + Sync>,
}

impl SmPassDao {
    pub fn new<P>(provide_aws_creds: P, region: &Region) -> impl PassDao
    where
        P: credential::ProvideAwsCredentials + Send + Sync + 'static,
    {
        SmPassDao {
            sm_client: Box::new(DefaultSmClient::new(provide_aws_creds, region)),
        }
    }
}

#[async_trait]
impl PassDao for SmPassDao {
    async fn create_password(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<Password> {
        let id = self.sm_client.create_secret_string(name, value, tags).await?;
        Ok(Password {
            id,
            name: name.to_string(),
            value: value.to_string(),
        })
    }

    async fn create_random_password(
        &self,
        name: &str,
        exclude_chars: Option<&str>,
        length: Option<&i64>,
        tags: Option<&[Tag]>,
    ) -> Result<Password> {
        let value = self.sm_client.get_random_password(exclude_chars, length).await?;
        self.create_password(name, &value, tags).await
    }

    async fn get_password(&self, id: &str) -> Result<Password> {
        self.sm_client.get_secret_string(id).await.map(|s| {
            Ok(Password {
                id: s.arn,
                name: s.name,
                value: s.value,
            })
        })?
    }

    async fn get_password_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<Password> {
        let all_filters: &[Filter] = &[
            &vec![("name".to_string(), vec![name.to_string()])],
            filters.unwrap_or_default(),
        ]
        .concat();
        let secrets = self.list_passwords(all_filters).await?;
        if (secrets.len() > 1) {
            // We update and delete secrets by name, we must enforce that get by name returns only 1 secret
            panic!("Received more than 1 secret when listing by name");
        }
        Ok(self
            .sm_client
            .get_secret_string(&secrets.iter().next().unwrap().id)
            .await?)
        .map(|s| Password {
            id: s.arn,
            name: s.name,
            value: s.value,
        })
    }

    async fn update_password(&self, id: &str, value: &str) -> Result<()> {
        self.sm_client.put_secret_string(id, value).await
    }

    async fn update_password_by_name(&self, name: &str, value: &str, filters: Option<&[Filter]>) -> Result<()> {
        let password = self.get_password_by_name(name, filters).await?;
        self.update_password(&password.id, value).await
    }

    async fn delete_password(&self, id: &str) -> Result<()> {
        self.sm_client.delete_secret(id).await
    }

    async fn delete_password_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<()> {
        let password = self.get_password_by_name(name, filters).await?;
        self.delete_password(&password.id).await
    }

    async fn list_passwords(&self, filters: &[Filter]) -> Result<Vec<PasswordDetails>> {
        let mut vec: Vec<PasswordDetails> = Vec::new();
        // Couldn't get Option<&str> to work
        let mut next_token: Option<String> = None;
        loop {
            // TODO: Use better error handling
            let (ss, nt) = self
                .sm_client
                .list_secrets(Some(filters), next_token.as_deref())
                .await?;
            let pds = &mut ss
                .into_iter()
                .map(|s| PasswordDetails {
                    id: s.arn,
                    name: s.name,
                })
                .collect();
            vec.append(pds);
            next_token = nt;
            if next_token.is_none() {
                break;
            }
        }
        Ok(vec)
    }
}
