// Represents a storage that can be used to store temporary data. (eg sessions)
use async_trait::async_trait;

#[async_trait]
pub trait TemporaryStorageProvider {
    async fn get(&self, key: String) -> Option<String>;
    async fn set(&self, key: String, value: String) -> bool;
    async fn delete(&self, key: String) -> bool;
    async fn drop_all(&self, value: String) -> bool;
}
