//! Ordered posting coordinates, never live installed-program authority.
use super::RelationalAuthoredWriteSelector;
use crate::validation::candidate_source::authored_history::ManagedSourceImpactKey;
use worth_execution::ChargedBytes;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum AuthoredWritePostingKey {
    Ordinary(RelationalAuthoredWriteSelector),
    ManagedSource(ManagedSourceImpactKey),
}
impl ChargedBytes for AuthoredWritePostingKey {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Ordinary(key) => key.additional_charged_bytes(),
            Self::ManagedSource(_) => 0,
        }
    }
}
