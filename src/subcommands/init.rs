use crate::traits::AsyncRunnable;
use async_trait::async_trait;
use clap::Clap;
use std::fmt::Write;
use std::{fs, process};

#[derive(Clap)]
pub struct Init {
    /// The store id
    #[clap()]
    id: String,
    /// The access key id for a user with STS perms
    #[clap()]
    aws_access_key_id: String,
    /// The secret access key for a user with STS perms
    #[clap()]
    aws_secret_access_key: String,
}

#[async_trait]
impl AsyncRunnable for Init {
    async fn run(self) {
        if self.id.is_empty() {
            eprintln!("aws-pass id must be non-empty");
            process::exit(1);
        }
        let aws_pass_dir = dirs::home_dir().unwrap().join(".aws-pass");
        if !aws_pass_dir.exists() {
            std::fs::create_dir(&aws_pass_dir).expect("Unable to create aws-pass dir");
        }
        let aws_pass_id = &aws_pass_dir.join(".id");
        if aws_pass_id.exists() {
            eprintln!("aws-pass store already initialized, not overwriting");
            process::exit(1);
        }

        // Write the provided id to the id file
        fs::write(aws_pass_id, self.id).expect("Unable to write id to file");
        let mut creds_str = String::new();
        write!(
            &mut creds_str,
            "aws_access_key_id={}\n",
            self.aws_access_key_id
        )
        .expect("Unable to write to buffer");
        write!(
            &mut creds_str,
            "aws_secret_access_key={}\n",
            self.aws_secret_access_key
        )
        .expect("Unable to write to buffer");
        fs::write(aws_pass_dir.join(".aws-credentials"), creds_str)
            .expect("Unable to write creds to file");
    }
}
