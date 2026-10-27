use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ContractId(String);

impl ContractId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}

impl From<&str> for ContractId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ContractId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<ContractId> for String {
    fn from(value: ContractId) -> Self {
        value.0
    }
}

impl Borrow<str> for ContractId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for ContractId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq<&str> for ContractId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for ContractId {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}
