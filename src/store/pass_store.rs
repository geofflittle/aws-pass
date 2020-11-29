use async_trait::async_trait;

#[async_trait]
pub trait PassStore {
    async fn init(&self);
    async fn list(&self, prefix: Option<&str>);
    async fn show(&self, name: &str);
    async fn insert(&self, name: &str);
    async fn edit(&self, name: &str);
    async fn generate(&self, name: &str, exclude_chars: Option<&str>, length: Option<&i64>);
    async fn remove(&self, name: &str);
}
