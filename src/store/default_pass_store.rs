use crate::dao::pass_dao::{Filter, PassDao, PassDaoErr, Password, PasswordDetails};

use super::pass_store::{PassStore, PassStoreErr};
use async_trait::async_trait;
use std::{
    io::{self, BufRead},
    path,
};

pub struct DefaultPassStore {
    store_dir: path::PathBuf,
    pass_dao: Box<dyn PassDao + Send + Sync>,
}

impl DefaultPassStore {
    pub fn new(
        store_dir: path::PathBuf,
        pass_dao: Box<dyn PassDao + Send + Sync>,
    ) -> Box<dyn PassStore> {
        Box::new(DefaultPassStore {
            store_dir,
            pass_dao,
        })
    }

    fn read_first_stdin_line(&self) -> String {
        let mut value = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut value).unwrap();
        value
    }
}

#[async_trait]
impl PassStore for DefaultPassStore {
    async fn init(&self) {
        // Check that creds file is in store_dir
        todo!()
    }

    async fn list(&self, prefix: Option<&str>) {
        let filters: Vec<Filter> = [
            vec![
                ("tag-key".to_string(), vec!["aws-pass".to_string()]),
                ("tag-value".to_string(), vec!["true".to_string()]),
            ],
            prefix
                .map(|p| vec![("name".to_string(), vec![p.to_string()])])
                .unwrap_or_default(),
        ]
        .concat();
        let passwords = self.pass_dao.list_passwords(&filters).await.unwrap();
        println!("{:?}", passwords);
    }

    async fn show(&self, id: &str) {
        let password = self.pass_dao.get_password(id).await.unwrap();
        println!("{:?}", password);
    }

    async fn insert(&self, name: &str) {
        let value = self.read_first_stdin_line();
        self.pass_dao.create_password(name, &value).await.unwrap();
    }

    async fn edit(&self, id: &str) {
        // Will need to get the password and open an editor
        todo!()
    }

    async fn generate(&self, name: &str) {
        todo!()
    }

    async fn remove(&self, id: &str) {
        self.pass_dao.delete_password(id).await.unwrap();
    }
}
