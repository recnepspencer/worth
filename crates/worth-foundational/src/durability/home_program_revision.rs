//! A recorded program boundary in one Home's original World attempt order.
use super::DurableCompositePublicationAttemptIdentity;
use crate::CanonicalDigestId;

/// Descriptive history, never permission to change a Home binding or replay a step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeProgramRevisionBoundary {
    predecessor: CanonicalDigestId,
    successor: CanonicalDigestId,
    through: DurableCompositePublicationAttemptIdentity,
}
impl HomeProgramRevisionBoundary {
    pub const fn from_recorded(
        predecessor: CanonicalDigestId,
        successor: CanonicalDigestId,
        through: DurableCompositePublicationAttemptIdentity,
    ) -> Self {
        Self {
            predecessor,
            successor,
            through,
        }
    }
    pub const fn predecessor(self) -> CanonicalDigestId {
        self.predecessor
    }
    pub const fn successor(self) -> CanonicalDigestId {
        self.successor
    }
    pub const fn through(self) -> DurableCompositePublicationAttemptIdentity {
        self.through
    }
}
