//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::computation_retention::retained_basis::RetainedBasis;

impl RetainedBasisToken {
    #[doc(hidden)]
    pub fn new_for_test(field0: RetainedBasis) -> Self {
        Self(field0)
    }
}
