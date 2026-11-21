//! Sealed test doors over this module's private members.
use super::*;
use worth_query_execution_lineage::facade::ports::{
    CommitAuthorityBindingInternalPort as _, RetainedGovernedInputInternalPort as _,
};

impl WorthQueryRecoveryBindingCurrentTruth {
    #[doc(hidden)]
    pub fn axis_probe_for_test(binding: &WorthQueryRecoveryHandleBinding) -> Self {
        Self::axis_probe(binding)
    }
}

impl WorthQueryRecoveryBindingCurrentTruth {
    #[doc(hidden)]
    pub fn check_for_test(
        &self,
        binding: &WorthQueryRecoveryHandleBinding,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        Self::check(self, binding)
    }
}
