use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Duration;
use scylla::{
    frame::value::ValueList, prepared_statement::PreparedStatement, FromRow, IntoTypedRows,
    Session, SessionBuilder,
};
use uuid::Uuid;

use crate::{structs::user::FullUser, traits::PersistentStorageProvider};

struct PreparedQueries {
    pub get_user: PreparedStatement,
    pub get_id_from_username: PreparedStatement,
    pub get_id_from_email: PreparedStatement,

    pub create_user: PreparedStatement,
    pub delete_user: PreparedStatement,

    pub get_user_from_username: PreparedStatement,
    pub get_user_from_email: PreparedStatement,

    pub verify_user: PreparedStatement,

    pub get_authentication_methods: PreparedStatement,
    pub update_authentication_method_value: PreparedStatement,
    pub remove_authentication_method: PreparedStatement,
}

pub struct ScyllaDataProvider {
    session: Session,
    prepared: PreparedQueries,
}

type UserRow = (
    Uuid,
    String,
    String,
    i64,
    Option<String>,
    Option<i16>,
    Option<HashMap<i16, String>>,
);

impl ScyllaDataProvider {
    pub async fn new() -> Self {
        // TODO: Get from .env
        let uri = "0.0.0.0:9042";
        let username = "cassandra";
        let password = "cassandra";

        let session = SessionBuilder::new()
            .known_node(uri)
            .user(username, password)
            .build()
            .await
            .expect("Failed to build scylla session");

        async fn prepare_query(session: &Session, query: &str) -> PreparedStatement {
            session.prepare(query).await.unwrap()
        }

        let prepared = PreparedQueries {
            get_user: prepare_query(
                &session,
                "SELECT id, username, email, created_at, verification_token, roles, authentication FROM accounts.users WHERE id = ?;",
            )
            .await,
            get_id_from_username: prepare_query(&session, "SELECT id FROM accounts.users WHERE username = ? LIMIT 1;").await,
            get_id_from_email: prepare_query(&session, "SELECT id FROM accounts.users WHERE email = ? LIMIT 1;").await,
            create_user: prepare_query(&session, "INSERT INTO accounts.users (id, username, email, created_at, authentication, verification_token) VALUES (?, ?, ?, ?, ?, ?);").await,
            delete_user: prepare_query(&session, "DELETE FROM accounts.users WHERE id = ?;").await,
            get_user_from_username: prepare_query(
                &session,
                "SELECT id, username, email, created_at, verification_token, roles, authentication FROM accounts.users WHERE username = ? LIMIT 1;",
            ).await,
            get_user_from_email: prepare_query(
                &session,
                "SELECT id, username, email, created_at, verification_token, roles, authentication FROM accounts.users WHERE email = ? LIMIT 1;",
            ).await,
            verify_user: prepare_query(&session, "UPDATE accounts.users SET verification_token = null WHERE id = ?;").await,

            get_authentication_methods: prepare_query(&session, "SELECT authentication FROM accounts.users WHERE id = ? LIMIT 1;").await,
            update_authentication_method_value: prepare_query(&session, "UPDATE accounts.users SET authentication[?] = ? WHERE id = ?;").await,
            remove_authentication_method: prepare_query(&session, "DELETE authentication[?] FROM accounts.users WHERE id = ?;").await,
        };

        ScyllaDataProvider { session, prepared }
    }

    async fn exists(&self, prepared: &PreparedStatement, args: impl ValueList) -> bool {
        self.session
            .execute(prepared, args)
            .await
            .unwrap()
            .rows
            .unwrap()
            .len()
            == 1
    }

    async fn get_first<T: FromRow>(
        &self,
        prepared: &PreparedStatement,
        args: impl ValueList,
    ) -> Option<T> {
        let res = self.session.execute(prepared, args).await;

        if let Ok(query) = res {
            if let Some(rows) = query.rows {
                if let Some(row) = rows.into_typed::<T>().next() {
                    return Some(row.unwrap());
                }
            }
        }

        None
    }

    async fn user_query(
        &self,
        prepared: &PreparedStatement,
        args: impl ValueList,
    ) -> Option<FullUser> {
        let res: Option<UserRow> = self.get_first(prepared, args).await;

        if let Some(row) = res {
            let (id, username, email, created_at, verification_token, roles, authentication) = row;

            return Some(FullUser {
                id,
                username,
                email,
                created_at: Duration::seconds(created_at),
                verification_token,
                roles: roles.unwrap_or_default() as usize,
                authentication: authentication.unwrap_or_default(),
            });
        }

        None
    }
}

#[async_trait]
impl PersistentStorageProvider for ScyllaDataProvider {
    async fn get_user_by_id(&self, id: Uuid) -> Option<FullUser> {
        self.user_query(&self.prepared.get_user, (id,)).await
    }

    async fn does_username_exist(&self, username: String) -> bool {
        self.exists(&self.prepared.get_id_from_username, (username,))
            .await
    }

    async fn does_email_exist(&self, email: String) -> bool {
        self.exists(&self.prepared.get_id_from_email, (email,))
            .await
    }

    async fn register_user(&self, user: FullUser) -> Result<(), String> {
        if self.does_username_exist(user.username.clone()).await {
            return Err("User already exists".to_string());
        } else if self.does_email_exist(user.email.clone()).await {
            return Err("Email already exists".to_string());
        }

        match self
            .session
            .execute(
                &self.prepared.create_user,
                (
                    user.id,
                    user.username,
                    user.email,
                    user.created_at.num_seconds(),
                    user.authentication,
                    user.verification_token,
                ),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to create user.".to_string()),
        }
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), String> {
        match self
            .session
            .execute(&self.prepared.delete_user, (id,))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to delete user".to_string()),
        }
    }

    async fn get_user_by_username(&self, username: String) -> Option<FullUser> {
        self.user_query(&self.prepared.get_user_from_username, (username,))
            .await
    }

    async fn get_user_by_email(&self, email: String) -> Option<FullUser> {
        self.user_query(&self.prepared.get_user_from_email, (email,))
            .await
    }

    async fn verify_user(&self, id: Uuid) -> Result<(), String> {
        match self
            .session
            .execute(&self.prepared.verify_user, (id,))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not verify user!".to_string()),
        }
    }

    async fn get_authentication_methods(&self, id: Uuid) -> Result<Vec<i16>, String> {
        match self
            .session
            .execute(&self.prepared.get_authentication_methods, (id,))
            .await
        {
            Ok(query) => {
                if let Some(rows) = query.rows {
                    if let Some(row) = rows.into_typed::<(HashMap<i16, String>,)>().next() {
                        let (methods,): (HashMap<i16, String>,) = row.unwrap();

                        return Ok(methods.keys().cloned().collect());
                    }
                }

                Err("Could not get authentication methods!".to_string())
            }
            Err(_) => Err("Could not get authentication methods!".to_string()),
        }
    }

    async fn update_authentication_method_value(
        &self,
        id: Uuid,
        method: i16,
        new_value: &str,
    ) -> Result<(), String> {
        match self
            .session
            .execute(
                &self.prepared.update_authentication_method_value,
                (method, new_value, id),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("{:?}", e);
                return Err("Could not update authentication method value!".to_string());
            }
        }
    }

    async fn remove_authentication_method(&self, id: Uuid, method: i16) -> Result<(), String> {
        match self
            .session
            .execute(&self.prepared.remove_authentication_method, (method, id))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not remove authentication method!".to_string()),
        }
    }
}
