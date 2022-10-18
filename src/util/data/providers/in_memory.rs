// Simple in memory data provider, to simulate a provider that interacts with
// a database.
//
// This should be used for testing purposes only. (not suitable for production)

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{structs::user::FullUser, traits::PersistentStorageProvider};

pub struct InMemoryDataProvider {
    users: HashMap<Uuid, FullUser>,
}

#[async_trait]
impl PersistentStorageProvider for InMemoryDataProvider {
    async fn register_user(&mut self, user: FullUser) -> Result<(), String> {
        self.users.insert(user.id, user);
        Ok(())
    }
}
