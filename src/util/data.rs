mod providers;

use std::sync::Mutex;

pub use providers::*;

pub struct Database {
    pub persistent: Mutex<PersistentStorage>,
    pub temporary: Mutex<TemporaryStorage>,
}

impl Database {
    pub fn new(persistent: PersistentStorage, temporary: TemporaryStorage) -> Self {
        Self {
            persistent: Mutex::new(persistent),
            temporary: Mutex::new(temporary),
        }
    }
}
