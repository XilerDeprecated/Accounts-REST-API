use async_trait::async_trait;
use ffly_rs::FireflyStream;

use crate::{constants::TTL, traits::TemporaryStorageProvider};

pub struct FireflyDataProvider {
    stream: FireflyStream,
}

impl FireflyDataProvider {
    pub async fn new() -> Self {
        // TODO: Fetch from env
        let address = "127.0.0.1:46600";
        let mut stream = FireflyStream::connect(address)
            .await
            .expect("Could not connect to Firefly database");

        stream.default_ttl = TTL;

        FireflyDataProvider { stream }
    }
}

#[async_trait]
impl TemporaryStorageProvider for FireflyDataProvider {
    async fn get(&self, key: String) -> Option<String> {
        match self.stream.get_value(&key).await {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    async fn set(&self, key: String, value: String) -> bool {
        self.stream.new(&key, &value).await.is_ok()
    }

    async fn delete(&self, key: String) -> bool {
        self.stream.drop(&key).await.is_ok()
    }

    async fn drop_all(&self, value: String) -> bool {
        self.stream.drop_values(&value).await.is_ok()
    }
}
