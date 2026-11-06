//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::provider::failure::RebaseVerificationFailure;

impl RebaseVerificationFailure {
    #[doc(hidden)]
    pub fn reason_for_test(&self) -> &Reason {
        Self::reason(self)
    }
}
