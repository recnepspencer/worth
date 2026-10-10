//! Withheld verification is not proof that the published input is unchanged.
use super::AcceptedCurrentCandidate;
pub(super) use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;

pub(super) enum CapturedDecisionInput {
    Uncaptured,
    Accepted(AcceptedCurrentCandidate),
    Withheld,
}
impl CapturedDecisionInput {
    pub(super) fn is_uncaptured(&self) -> bool {
        matches!(self, Self::Uncaptured)
    }
    pub(super) fn capture(
        candidate: &Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
    ) -> Self {
        match candidate {
            Ok(Some(candidate)) => Self::Accepted(candidate.clone()),
            Ok(None) => Self::Uncaptured,
            Err(_) => Self::Withheld,
        }
    }
    pub(super) fn from_result(
        candidate: Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
    ) -> Self {
        match candidate {
            Ok(Some(candidate)) => Self::Accepted(candidate),
            Ok(None) => Self::Uncaptured,
            Err(_) => Self::Withheld,
        }
    }
}
