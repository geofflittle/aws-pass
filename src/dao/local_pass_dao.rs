use super::pass_dao::{PassDao, PassDaoErr, Password};
use async_trait::async_trait;

pub struct LocalPassDao {}

impl LocalPassDao {
    pub fn new() -> impl PassDao {
        LocalPassDao {}
    }
}

#[async_trait]
impl PassDao for LocalPassDao {
    async fn create_password(&self, name: &str, value: &str) -> Result<String, PassDaoErr> {
        todo!()
    }

    async fn get_password(&self, id: &str) -> Result<Password, PassDaoErr> {
        todo!()
    }

    async fn update_password(&self, id: &str, value: &str) -> Result<(), PassDaoErr> {
        todo!()
    }

    async fn delete_password(&self, id: &str) -> Result<(), PassDaoErr> {
        todo!()
    }

    async fn list_passwords(
        &self,
        filters: &[super::pass_dao::Filter],
    ) -> Result<Vec<super::pass_dao::PasswordDetails>, PassDaoErr> {
        todo!()
    }
}
