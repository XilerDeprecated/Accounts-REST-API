mod in_memory;
mod scylla;

// This allows us to easily swap the data provider.
// pub use in_memory::InMemoryDataProvider as PersistentStorage;
pub use self::scylla::ScyllaDataProvider as PersistentStorage;
pub use in_memory::InMemoryDataProvider as TemporaryStorage;
