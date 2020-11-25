use async_trait::async_trait;

#[async_trait]
pub trait AsyncRunnable {
    async fn run(self);
}
