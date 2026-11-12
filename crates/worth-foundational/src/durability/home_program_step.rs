//! Exact canonical transition attribution from an admitted Home intent.
use super::CompositeAttemptAssociation;
use crate::CanonicalDigestId;
/// Descriptive original membership, never permission to execute or replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionProgramStep {
    original: CompositeAttemptAssociation,
    recorded: CanonicalDigestId,
    activation: CanonicalDigestId,
}
impl HomeProgramTransitionProgramStep {
    pub fn from_recorded(
        original: CompositeAttemptAssociation,
        recorded: CanonicalDigestId,
        activation: CanonicalDigestId,
    ) -> Self {
        Self {
            original,
            recorded,
            activation,
        }
    }
    pub fn original(&self) -> &CompositeAttemptAssociation {
        &self.original
    }
    pub const fn recorded(&self) -> CanonicalDigestId {
        self.recorded
    }
    pub const fn activation(&self) -> CanonicalDigestId {
        self.activation
    }
}
