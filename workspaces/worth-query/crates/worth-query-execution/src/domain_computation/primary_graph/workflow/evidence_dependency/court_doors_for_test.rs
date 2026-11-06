//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowEvidenceDependencyKind {
    #[doc(hidden)]
    pub fn decode_for_test(value: &str) -> Option<Self> {
        Self::decode(value)
    }
}

impl WorkflowEvidenceDependencyKind {
    #[doc(hidden)]
    pub fn encode_for_test(self) -> &'static str {
        Self::encode(self)
    }
}
