//! `output_family_identity`'s vocabulary, moved whole from `primary_graph::output_family_identity`: its root file's items here, one child per other source file.

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputFamilyIdentity(Cow<'static, str>);

impl OutputFamilyIdentity {
    pub(in crate::domain_computation::primary_graph) const fn declared(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    pub(in crate::domain_computation::primary_graph) fn as_str(&self) -> &str {
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

impl OutputFamilyIdentity {
    #[doc(hidden)]
    pub fn declared_for_test(identity: &'static str) -> Self {
        Self::declared(identity)
    }
}
use std::borrow::Borrow;
use std::borrow::Cow;
