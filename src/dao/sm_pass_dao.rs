use super::pass_dao::{Filter, PassDao, Password, PasswordDetails, Tag};
use crate::client::{default_sm_client::DefaultSmClient, sm_client::SmClient};
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
    async fn create_password(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String> {
        self.sm_client.create_secret_string(name, value, tags).await
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
        self.sm_client.get_secret_string_by_name(name, filters).await.map(|s| {
            Ok(Password {
                id: s.arn,
                name: s.name,
                value: s.value,
            })
        })?
    }

    async fn update_password(&self, id: &str, value: &str) -> Result<()> {
        self.sm_client.put_secret_string(id, value).await
    }

    async fn update_password_by_name(&self, id: &str, value: &str, filters: Option<&[Filter]>) -> Result<()> {
        self.sm_client.put_secret_string_by_name(id, value, filters).await
    }

    async fn delete_password(&self, id: &str) -> Result<()> {
        self.sm_client.delete_secret(id).await
    }

    async fn delete_password_by_name(&self, id: &str, filters: Option<&[Filter]>) -> Result<()> {
        self.sm_client.delete_secret_by_name(id, filters).await
    }

    async fn list_passwords(&self, filters: &[Filter]) -> Result<Vec<PasswordDetails>> {
        self.sm_client
            .list_all_secrets(Some(filters))
            .await
            .map(|s| s.into_iter().map(|s| PasswordDetails { name: s.name }).collect())
    }
}
