use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait PassDao {
    async fn create_password(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String>;
    async fn get_password(&self, id: &str) -> Result<Password>;
    async fn get_password_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<Password>;
    async fn update_password(&self, id: &str, value: &str) -> Result<()>;
    async fn update_password_by_name(&self, name: &str, value: &str, filters: Option<&[Filter]>) -> Result<()>;
    async fn delete_password(&self, id: &str) -> Result<()>;
    async fn delete_password_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<()>;
    async fn list_passwords(&self, filters: &[Filter]) -> Result<Vec<PasswordDetails>>;
}

pub type Tag = (String, String);
pub type Filter = (String, Vec<String>);

#[derive(Debug)]
pub struct PasswordDetails {
    pub name: String,
}

#[derive(Debug)]
pub struct Password {
    pub id: String,
    pub name: String,
    pub value: String,
}
