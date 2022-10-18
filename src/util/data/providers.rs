mod in_memory;

// This allows us to easily swap the data provider.
pub use in_memory::InMemoryDataProvider as PersistentStorage;
pub use in_memory::InMemoryDataProvider as TemporaryStorage;
