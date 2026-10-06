use std::marker::PhantomData;

const MAX_ENTITY_KEY_BYTES: usize = 512;

/// Caller-chosen key for an entity created by an operation or seeded at
/// bootstrap.
///
/// Build with `new`; the key must be non-empty, have no surrounding
/// whitespace or control characters, and be at most 512 bytes. It names the
/// entity within its kind and grants nothing.
pub struct WorthQueryApplicationEntityKey<Schema, Entity> {
    value: String,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> WorthQueryApplicationEntityKey<Schema, Entity> {
    pub fn new(value: impl Into<String>) -> Result<Self, WorthQueryApplicationEntityKeyDenial> {
        let value = value.into();
        let invalid = value.is_empty()
            || value.trim() != value
            || value.chars().any(char::is_control)
            || value.len() > MAX_ENTITY_KEY_BYTES;
        if invalid {
            return Err(WorthQueryApplicationEntityKeyDenial);
        }
        Ok(Self {
            value,
            _marker: PhantomData,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub(super) fn owned_allocation_capacity_bytes(&self) -> usize {
        self.value.capacity()
    }

    pub(crate) fn into_string(self) -> String {
        self.value
    }
}

impl<Schema, Entity> std::fmt::Debug for WorthQueryApplicationEntityKey<Schema, Entity> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("WorthQueryApplicationEntityKey")
            .field(&self.value)
            .finish()
    }
}

/// Refusal to build an entity key because the value is empty, padded, contains
/// control characters, or is longer than 512 bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationEntityKeyDenial;

impl std::fmt::Display for WorthQueryApplicationEntityKeyDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("application entity key is not a bounded canonical identifier")
    }
}

impl std::error::Error for WorthQueryApplicationEntityKeyDenial {}
