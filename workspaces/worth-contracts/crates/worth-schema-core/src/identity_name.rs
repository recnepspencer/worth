use crate::naming::{InvalidName, Name};

/// A validated [`Name`] used as the name of an [`Identity`](crate::identity::Identity).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdentityName(Name);

impl IdentityName {
    /// Validate a raw string as an identity name.
    ///
    /// Fails with [`InvalidName`] under the same rules as [`Name::new`].
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidName> {
        Ok(Self(Name::new(raw)?))
    }

    /// The underlying name.
    pub fn as_name(&self) -> &Name {
        &self.0
    }

    /// The name as a string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
