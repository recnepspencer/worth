//! A published member runs again only on a different semantic source or accepted
//! input that compares as changed. Missing or withheld comparisons refuse another
//! attempt; readers own currentness verification.
use super::AcceptedCurrentCandidate;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;

pub enum CapturedDecisionInput {
    Uncaptured,
    Accepted(AcceptedCurrentCandidate),
    // A withheld first capture cannot be replaced by a later accepted candidate.
    Withheld,
}
impl CapturedDecisionInput {
    pub fn is_uncaptured(&self) -> bool {
        matches!(self, Self::Uncaptured)
    }
    pub fn capture(
        candidate: &Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
    ) -> Self {
        match candidate {
            Ok(Some(candidate)) => Self::Accepted(candidate.clone()),
            Ok(None) => Self::Uncaptured,
            Err(_) => Self::Withheld,
        }
    }
    pub fn from_result(
        candidate: Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
    ) -> Self {
        match candidate {
            Ok(Some(candidate)) => Self::Accepted(candidate),
            Ok(None) => Self::Uncaptured,
            Err(_) => Self::Withheld,
        }
    }
}
