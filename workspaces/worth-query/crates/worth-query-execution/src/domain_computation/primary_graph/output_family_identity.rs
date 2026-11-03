//! Declared output-family identity is the sole family ordering axis.
use std::borrow::{Borrow, Cow};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct OutputFamilyIdentity(Cow<'static, str>);

impl OutputFamilyIdentity {
    pub(super) const fn declared(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for OutputFamilyIdentity {
    fn from(identity: String) -> Self {
        Self(Cow::Owned(identity))
    }
}

impl Borrow<str> for OutputFamilyIdentity {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
