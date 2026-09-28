use crate::identity_name::IdentityName;
use crate::naming::InvalidName;

/// The identity a schema element carries: either none, or a validated name.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Identity {
    /// The element has no name.
    Anonymous,
    /// The element is named.
    Named(IdentityName),
}

impl Identity {
    /// An identity with no name.
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    /// An identity with the given name.
    pub fn named(name: IdentityName) -> Self {
        Self::Named(name)
    }

    /// Parse a raw string as a named identity.
    ///
    /// Fails with [`InvalidName`] if the string is not a valid [`Name`](crate::naming::Name).
    pub fn parse(raw: &str) -> Result<Self, InvalidName> {
        Ok(Self::Named(IdentityName::new(raw)?))
    }
}
