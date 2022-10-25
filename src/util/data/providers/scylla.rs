use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Duration;
use scylla::{
    frame::value::{Time, ValueList},
    prepared_statement::PreparedStatement,
    IntoTypedRows, Session, SessionBuilder,
};
use uuid::Uuid;

use crate::{structs::user::FullUser, traits::PersistentStorageProvider};

struct PreparedQueries {
    pub get_user: PreparedStatement,
    pub get_id_from_username: PreparedStatement,
    pub get_id_from_email: PreparedStatement,
    pub create_user: PreparedStatement,
    pub delete_user: PreparedStatement,
}

pub struct ScyllaDataProvider {
    session: Session,
    prepared: PreparedQueries,
}

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
}

#[async_trait]
impl PersistentStorageProvider for ScyllaDataProvider {
    async fn get_user_by_id(&self, id: Uuid) -> Option<FullUser> {
        let res = self.session.execute(&self.prepared.get_user, (id,)).await;

        if let Ok(query) = res {
            if let Some(rows) = query.rows {
                if let Some(row) = rows
                    .into_typed::<(
                        Uuid,
                        String,
                        String,
                        Duration,
                        Option<String>,
                        Option<i16>,
                        Option<HashMap<i16, String>>,
                    )>()
                    .next()
                {
                    let (
                        id,
                        username,
                        email,
                        created_at,
                        verification_token,
                        roles,
                        authentication,
                    ) = row.unwrap();

                    return Some(FullUser {
                        id,
                        username,
                        email,
                        created_at,
                        verification_token,
                        roles: roles.unwrap_or_default() as usize,
                        authentication: authentication.unwrap_or_default(),
                    });
                }
            }
        }

        None
    }

    async fn does_username_exist(&self, username: String) -> bool {
        self.exists(&self.prepared.get_id_from_username, (username,))
            .await
    }

    async fn does_email_exist(&self, email: String) -> bool {
        self.exists(&self.prepared.get_id_from_email, (email,))
            .await
    }

    async fn register_user(&mut self, user: FullUser) -> Result<(), String> {
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
                    Time(user.created_at),
                    user.authentication,
                    user.verification_token,
                ),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to create user".to_string()),
        }
    }

    async fn delete_user(&mut self, id: Uuid) -> Result<(), String> {
        match self
            .session
            .execute(&self.prepared.delete_user, (id,))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to delete user".to_string()),
        }
    }
}
