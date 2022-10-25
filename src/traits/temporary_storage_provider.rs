// Represents a storage that can be used to store temporary data. (eg sessions)
use async_trait::async_trait;

#[async_trait]
pub trait TemporaryStorageProvider {
    async fn get(&mut self, key: String) -> Option<String>;
    async fn set(&mut self, key: String, value: String) -> bool;
    async fn delete(&mut self, key: String) -> bool;
    async fn drop_all(&mut self, value: String) -> bool;
}
