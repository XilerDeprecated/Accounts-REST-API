// Simple in memory data provider, to simulate a provider that interacts with
// a database.
//
// This should be used for testing purposes only. (not suitable for production)

use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    structs::user::FullUser,
    traits::{PersistentStorageProvider, TemporaryStorageProvider},
};

pub struct InMemoryDataProvider {
    users: HashMap<Uuid, FullUser>,
    sessions: HashMap<String, String>,
}

impl InMemoryDataProvider {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

#[async_trait]
impl PersistentStorageProvider for InMemoryDataProvider {
    async fn does_username_exist(&self, username: String) -> bool {
        self.users
            .values()
            .any(|user| user.username == username.to_string())
    }

    async fn does_email_exist(&self, email: String) -> bool {
        self.users
            .values()
            .any(|user| user.email == email.to_string())
    }

    async fn register_user(&mut self, user: FullUser) -> Result<(), String> {
        if self.does_username_exist(user.username.clone()).await {
            return Err("User already exists".to_string());
        } else if self.does_email_exist(user.email.clone()).await {
            return Err("Email already exists".to_string());
        }

        self.users.insert(user.id, user);
        Ok(())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Option<FullUser> {
        self.users.get(&id).cloned()
    }

    async fn delete_user(&mut self, id: Uuid) -> Result<(), String> {
        let user = self.get_user_by_id(id).await;
        match user {
            Some(user) => {
                self.users.remove(&user.id);
                Ok(())
            }
            None => Err("User does not exist".to_string()),
        }
    }
}

#[async_trait]
impl TemporaryStorageProvider for InMemoryDataProvider {
    async fn get(&mut self, key: String) -> Option<String> {
        self.sessions.get(&key).cloned()
    }

    async fn set(&mut self, key: String, value: String) -> bool {
        self.sessions.insert(key, value);
        true
    }

    async fn delete(&mut self, key: String) -> bool {
        self.sessions.remove(&key);
        true
    }

    async fn get_by_value(&self, value: String) -> Vec<String> {
        self.sessions
            .iter()
            .filter(|(_, v)| **v == value)
            .map(|(k, _)| k.to_string())
            .collect()
    }

    async fn drop_all(&mut self, keys: Vec<String>) -> bool {
        for key in keys {
            self.delete(key).await;
        }
        true
    }
}
