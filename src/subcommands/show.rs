use crate::traits::AsyncRunnable;
use async_trait::async_trait;
use clap::Clap;

#[derive(Clap)]
pub struct Show {}

#[async_trait]
impl AsyncRunnable for Show {
    async fn run(self) {
        todo!()
    }
}
