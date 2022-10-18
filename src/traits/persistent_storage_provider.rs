// Represents a persistent storage provider, such as MySQL, Postgres, ScyllaDB, etc.
// This does not mean that the data stored on this provider is permanent, but that
// it stores data that should not expire after a short amount of time. (eg session keys should not
// be stored here)
use async_trait::async_trait;

use crate::structs::user::FullUser;

#[async_trait]
pub trait PersistentStorageProvider {
    async fn register_user(&mut self, user: FullUser) -> Result<(), String>;
}
