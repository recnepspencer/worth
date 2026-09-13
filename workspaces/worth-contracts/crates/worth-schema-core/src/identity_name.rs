use crate::naming::{InvalidName, Name};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdentityName(Name);

impl IdentityName {
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidName> {
        Ok(Self(Name::new(raw)?))
    }

    pub fn as_name(&self) -> &Name {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
