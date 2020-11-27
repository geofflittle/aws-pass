use super::pass_dao::{Filter, PassDao, PassDaoErr, Password, PasswordDetails};
use crate::client::{default_sm_client::DefaultSMClient, sm_client::SMClient};
use async_trait::async_trait;
use rusoto_core::Region;

pub struct SMPassDao {
    sm_client: Box<dyn SMClient + Send + Sync>,
}

impl SMPassDao {
    pub fn new(region: Region) -> impl PassDao {
        SMPassDao {
            sm_client: Box::new(DefaultSMClient::new(region)),
        }
    }
}

#[async_trait]
impl PassDao for SMPassDao {
    async fn create_password(
        &self,
        name: &str,
        value: &str,
    ) -> Result<String, PassDaoErr> {
        // TODO: Use better error handling
        self.sm_client
            .create_secret_string(name, value)
            .await
            .map(|id| Ok(id))
            .unwrap()
    }

    async fn get_password(&self, id: &str) -> Result<Password, PassDaoErr> {
        // TODO: Use better error handling
        self.sm_client
            .get_secret_string(id)
            .await
            .map(|s| {
                Ok(Password {
                    name: s.name,
                    value: s.value,
                })
            })
            .unwrap()
    }

    async fn update_password(
        &self,
        id: &str,
        value: &str,
    ) -> Result<(), PassDaoErr> {
        // TODO: Use better error handling
        self.sm_client
            .put_secret_string(id, value)
            .await
            .map(|_| Ok(()))
            .unwrap()
    }

    async fn delete_password(&self, id: &str) -> Result<(), PassDaoErr> {
        // TODO: Use better error handling
        self.sm_client
            .delete_secret(id)
            .await
            .map(|_| Ok(()))
            .unwrap()
    }

    async fn list_passwords(
        &self,
        filters: &[Filter],
    ) -> Result<Vec<PasswordDetails>, PassDaoErr> {
        // TODO: Use better error handling
        Ok(self
            .sm_client
            .list_all_secrets(Some(filters))
            .await
            .unwrap()
            .into_iter()
            .map(|sd| PasswordDetails { name: sd.name })
            .collect())
    }
}
