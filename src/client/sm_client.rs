use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SmClient {
    /// Creates a secret string given a **name** and a **value**, returns the
    /// created secret's **arn**.
    async fn create_secret_string(&self, name: &str, value: &str, tags: Option<&[Tag]>) -> Result<String>;

    /// Deletes the secret for the given **arn**.
    async fn delete_secret(&self, arn: &str) -> Result<()>;

    /// Deletes the secret for the given **arn**.  Secrets are not id'd by their
    /// name so all secrets must be listed and their names checked for
    /// equality.  Optional **filters** can be provided for optimizing the
    /// listing.
    async fn delete_secret_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<()>;

    /// Describes the secret for the given **arn**.
    async fn describe_secret(&self, arn: &str) -> Result<SecretDetails>;

    /// Gets the secret for the given **arn**.
    async fn get_secret_string(&self, arn: &str) -> Result<SecretString>;

    /// Gets the secret for the given **name**.  Secrets are not id'd by their
    /// name so all secrets must be listed and their names checked for
    /// equality.  Optional **filters** can be provided for optimizing the
    /// listing.
    async fn get_secret_string_by_name(&self, name: &str, filters: Option<&[Filter]>) -> Result<SecretString>;

    /// Lists secrets for the given optional **filters** and **next_token**.
    /// Returns a page with its results and next token.
    async fn list_secrets(&self, filters: Option<&[Filter]>, next_token: Option<&str>) -> Result<SecretsPage>;

    /// Lists all the secrets given optional **filters**.  Note: Be careful,
    /// you may not actually want to list ***ALL*** secrets.
    async fn list_all_secrets(&self, filters: Option<&[Filter]>) -> Result<Vec<SecretDetails>>;

    /// Puts a secret string for the given **arn** and secret **value**.
    async fn put_secret_string(&self, arn: &str, value: &str) -> Result<()>;

    /// Puts a secret string for the given **name**, **value**, and optional
    /// **filters**.  Secrets are not id'd by their name so all secrets must be
    /// listed and their names checked for equality.  Optional **filters** can
    /// be provided for optimizing the listing.
    async fn put_secret_string_by_name(&self, name: &str, value: &str, filters: Option<&[Filter]>) -> Result<()>;
}

pub type SecretsPage = (Vec<SecretDetails>, Option<String>);
pub type Tag = (String, String);
pub type Filter = (String, Vec<String>);

#[derive(Debug)]
pub struct SecretDetails {
    pub arn: String,
    pub name: String,
    pub tags: Vec<(String, String)>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct SecretString {
    pub arn: String,
    pub name: String,
    pub value: String,
}
