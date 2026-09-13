use crate::identity_name::IdentityName;
use crate::naming::InvalidName;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Identity {
    Anonymous,
    Named(IdentityName),
}

impl Identity {
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    pub fn named(name: IdentityName) -> Self {
        Self::Named(name)
    }

    pub fn parse(raw: &str) -> Result<Self, InvalidName> {
        Ok(Self::Named(IdentityName::new(raw)?))
    }
}
