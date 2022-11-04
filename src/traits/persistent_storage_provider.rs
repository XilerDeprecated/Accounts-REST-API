// Represents a persistent storage provider, such as MySQL, Postgres, ScyllaDB, etc.
// This does not mean that the data stored on this provider is permanent, but that
// it stores data that should not expire after a short amount of time. (eg session keys should not
// be stored here)
use async_trait::async_trait;
use uuid::Uuid;

use crate::structs::user::FullUser;

#[async_trait]
pub trait PersistentStorageProvider {
    async fn get_user_by_username(&self, username: String) -> Option<FullUser>;
    async fn get_user_by_email(&self, email: String) -> Option<FullUser>;
    async fn get_user_by_id(&self, id: Uuid) -> Option<FullUser>;

    async fn does_username_exist(&self, username: String) -> bool;
    async fn does_email_exist(&self, email: String) -> bool;

    async fn register_user(&self, user: FullUser) -> Result<(), String>;
    async fn delete_user(&self, id: Uuid) -> Result<(), String>;

    async fn verify_user(&self, id: Uuid) -> Result<(), String>;

    async fn remove_authentication_method(&self, id: Uuid, method: i16) -> Result<(), String>;
    async fn update_authentication_method_value(
        &self,
        id: Uuid,
        method: i16,
        new_value: String,
    ) -> Result<(), String>;
    async fn get_authentication_methods(&self, id: Uuid) -> Result<Vec<i16>, String>;
}
