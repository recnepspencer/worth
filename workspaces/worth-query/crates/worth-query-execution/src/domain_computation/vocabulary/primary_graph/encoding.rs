//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow::actor::encoding`: the items below and every impl they own.
use serde::Serialize;
use serde::Serializer;
use worth_execution::ExecutionImmutableBytes;

/// Clones retain the same text allocation and its original payer. No owned
/// String can be extracted; an independent field copy requires fresh admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundedActorText(ExecutionImmutableBytes);

impl FundedActorText {
    pub(in crate::domain_computation) fn bytes(&self) -> &ExecutionImmutableBytes {
        &self.0
    }
    pub(in crate::domain_computation) fn as_str(&self) -> &str {
        std::str::from_utf8(self.0.bytes()).expect("serde JSON emits UTF-8")
    }
}

impl Serialize for FundedActorText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl std::ops::Deref for FundedActorText {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for FundedActorText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FundedActorText {
    #[doc(hidden)]
    pub fn as_str_for_test(&self) -> &str {
        Self::as_str(self)
    }
}
