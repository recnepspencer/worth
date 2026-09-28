use std::marker::PhantomData;

const MAX_PRINCIPAL_KEY_BYTES: usize = 512;

/// Typed bootstrap identity for one application principal entity.
///
/// This is descriptive client identity, not principal or operation authority.
pub struct WorthQueryApplicationPrincipalKey<Schema, Principal> {
    value: String,
    _marker: PhantomData<fn() -> (Schema, Principal)>,
}

impl<Schema, Principal> WorthQueryApplicationPrincipalKey<Schema, Principal> {
    pub fn new(value: impl Into<String>) -> Result<Self, WorthQueryApplicationPrincipalKeyDenial> {
        let value = value.into();
        let invalid = value.is_empty()
            || value.trim() != value
            || value.chars().any(char::is_control)
            || value.len() > MAX_PRINCIPAL_KEY_BYTES;
        if invalid {
            return Err(WorthQueryApplicationPrincipalKeyDenial);
        }
        Ok(Self {
            value,
            _marker: PhantomData,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<Schema, Principal> std::fmt::Debug for WorthQueryApplicationPrincipalKey<Schema, Principal> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("WorthQueryApplicationPrincipalKey")
            .field(&self.value)
            .finish()
    }
}

/// Refusal to build a principal key because the value is empty, padded,
/// contains control characters, or is longer than 512 bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationPrincipalKeyDenial;

impl std::fmt::Display for WorthQueryApplicationPrincipalKeyDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("application principal key is not a bounded canonical identifier")
    }
}

impl std::error::Error for WorthQueryApplicationPrincipalKeyDenial {}
