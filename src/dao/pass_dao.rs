use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait PassDao {
    async fn create_password(
        &self,
        name: &str,
        value: &str,
    ) -> Result<String, PassDaoErr>;
    async fn get_password(&self, id: &str) -> Result<Password, PassDaoErr>;
    async fn update_password(
        &self,
        id: &str,
        value: &str,
    ) -> Result<(), PassDaoErr>;
    async fn delete_password(&self, id: &str) -> Result<(), PassDaoErr>;
    async fn list_passwords(
        &self,
        filters: &[Filter],
    ) -> Result<Vec<PasswordDetails>, PassDaoErr>;
}

pub type Filter = (String, Vec<String>);

#[derive(Debug)]
pub struct PasswordDetails {
    pub name: String,
}

#[derive(Debug)]
pub struct Password {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub enum PassDaoErr {
    PasswordAlreadyExists(String),
    IOError(io::Error),
    NotFound,
    OtherError,
}
