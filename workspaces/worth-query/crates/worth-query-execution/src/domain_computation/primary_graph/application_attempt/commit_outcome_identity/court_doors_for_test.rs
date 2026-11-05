//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity;


impl WorthQueryApplicationCommitOutcomeIdentity {
    #[doc(hidden)]
    pub fn restore_for_test(value: u64) -> Option<Self> {
        Self::restore(value)
    }
}
