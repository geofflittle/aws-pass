use super::pass_dao::{Filter, PassDao, PassDaoErr, Password, PasswordDetails, Tag};
use crate::client::{default_sm_client::DefaultSmClient, sm_client::SmClient};
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
    async fn create_password(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String, PassDaoErr> {
        self.sm_client
            .create_secret_string(name, value, tags)
            .await
            .map(|id| Ok(id))
            .expect("Unable to create secret string")
    }

    async fn get_password(&self, id: &str) -> Result<Password, PassDaoErr> {
        self.sm_client
            .get_secret_string(id)
            .await
            .map(|s| {
                Ok(Password {
                    name: s.name,
                    value: s.value,
                })
            })
            .expect("Unable to get secret string")
    }

    async fn get_password_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<Password, PassDaoErr> {
        self.sm_client
            .get_secret_string_by_name(name, filters)
            .await
            .map(|s| {
                Ok(Password {
                    name: s.name,
                    value: s.value,
                })
            })
            .expect("Unable to get secret string")
    }

    async fn update_password(&self, id: &str, value: &str) -> Result<(), PassDaoErr> {
        self.sm_client
            .put_secret_string(id, value)
            .await
            .map(|_| Ok(()))
            .expect("Unable to put secret string")
    }

    async fn update_password_by_name(
        &self,
        id: &str,
        value: &str,
        filters: Option<&[Filter]>,
    ) -> Result<(), PassDaoErr> {
        self.sm_client
            .put_secret_string_by_name(id, value, filters)
            .await
            .map(|_| Ok(()))
            .expect("Unable to put secret string")
    }

    async fn delete_password(&self, id: &str) -> Result<(), PassDaoErr> {
        self.sm_client
            .delete_secret(id)
            .await
            .map(|_| Ok(()))
            .expect("Unable to delete secret")
    }

    async fn delete_password_by_name(&self, id: &str, filters: Option<&[Filter]>) -> Result<(), PassDaoErr> {
        self.sm_client
            .delete_secret_by_name(id, filters)
            .await
            .map(|_| Ok(()))
            .expect("Unable to delete secret")
    }

    async fn list_passwords(&self, filters: &[Filter]) -> Result<Vec<PasswordDetails>, PassDaoErr> {
        Ok(self
            .sm_client
            .list_all_secrets(Some(filters))
            .await
            .expect("Unable to list all secrets"))
        .map(|pds| pds.into_iter().map(|s| PasswordDetails { name: s.name }).collect())
    }
}
